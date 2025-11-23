use crate::server::auth::UserPrincipal;
use crate::server::db::models::user;
use crate::server::db::models::user::{User, UserBuilder};
use crate::server::db::models::user_auth::{IdentityType, UserAuth, UserAuthBuilder};
use crate::server::db::{Pool, tools};
use crate::server::user::UserListReq;
use crate::server::user::request::{LoginReq, UpdatePasswordReq, UserAddReq, UserUpdateReq};
use crate::server::user::response::{
    LoginRes, OtherInfo, UserBaseInfo, UserCenterRes, UserListRes,
};
use anyhow::bail;
use cache::caches::CacheKey;
use common::id;
use protocol::common::req::{IdsReq, Pagination};
use protocol::common::res::PageRes;
use rbs::value;
use std::time::Duration;
use validator::Validate;

pub(crate) async fn login(req: LoginReq) -> anyhow::Result<LoginRes> {
    let user_id = match req.login_type {
        1 => {
            let user_auth = UserAuth::select_by_map(
                Pool::get()?,
                value! {
                    "identity": req.identity,
                },
            )
            .await?;
            if user_auth.is_empty() {
                bail!("用户名或密码不正确");
            }
            // 匹配密码，只要有一个匹配即可
            user_auth
                .into_iter()
                .find(|auth| {
                    bcrypt::verify(&req.secret, &auth.secret.clone().unwrap_or_default())
                        .unwrap_or(false)
                })
                .map(|auth| auth.user_id.unwrap())
                .ok_or_else(|| anyhow::anyhow!("用户名或密码不正确"))?
        }
        _ => bail!("登录方式不支持"),
    };

    let user = User::select_by_map(
        Pool::get()?,
        value! {
            "id": user_id,
        },
    )
    .await?;
    if user.is_empty() {
        bail!("用户名或密码不正确");
    }
    let user = user.first().unwrap();
    let user_principal = UserPrincipal {
        id: user.id.unwrap(),
        nickname: user.nickname.clone(),
        token: None,
    };
    let token = uuid::Uuid::new_v4().to_string();
    cache::set(
        CacheKey::UserToken(token.clone()).to_string(),
        &user_principal,
        Some(Duration::from_secs(3600 * 24).as_secs()),
    )
    .await?;

    // 更新登录时间
    let tx = Pool::get()?;
    User::update_by_map(
        tx,
        &UserBuilder::default()
            .last_login_time(Some(tools::now()))
            .build()?,
        value! {
            "id": user.id,
        },
    )
    .await?;

    Ok(LoginRes { token })
}

pub(crate) async fn logout(user: UserPrincipal) -> anyhow::Result<()> {
    cache::remove(&CacheKey::UserToken(user.token.unwrap_or_default()).to_string()).await?;
    Ok(())
}

pub(crate) async fn update_password(
    req: UpdatePasswordReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    let password = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    let user_auth = UserAuthBuilder::default()
        .secret(Some(password))
        .update_time(Some(tools::now()))
        .build()?;
    UserAuth::update_by_map(
        Pool::get()?,
        &user_auth,
        value! {
            "user_id": user.id,
        },
    )
    .await?;
    Ok(())
}

pub(crate) async fn center(user: UserPrincipal) -> anyhow::Result<UserCenterRes> {
    let user = User::select_by_map(
        Pool::get()?,
        value! {
            "id": user.id,
        },
    )
    .await?;
    if user.is_empty() {
        bail!("User does not exist");
    }
    let user = user.first().unwrap();
    let username_auth = UserAuth::select_by_map(
        Pool::get()?,
        value! {
            "user_id": user.id,
            "type": IdentityType::Username as i8,
        },
    )
    .await?;
    let email_auth = UserAuth::select_by_map(
        Pool::get()?,
        value! {
            "user_id": user.id,
            "type": IdentityType::Email as i8,
        },
    )
    .await?;
    if username_auth.is_empty() && email_auth.is_empty() {
        bail!("User does not exist");
    }
    let username_auth = username_auth.first().unwrap();
    let email_auth = email_auth.first().unwrap();

    Ok(UserCenterRes {
        id: user.id.unwrap(),
        base_info: UserBaseInfo {
            username: username_auth.identity.clone().unwrap(),
            nickname: user.nickname.clone(),
            avatar: user.avatar.clone(),
            email: email_auth.identity.clone().unwrap(),
        },
        other: OtherInfo {
            password_has_set: username_auth.secret.is_some(),
        },
    })
}

mod tests {
    #[test]
    pub fn gen_password() {
        let password = "admin";
        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        println!("{}", hashed);
    }
}

pub async fn list(req: UserListReq, _user: UserPrincipal) -> anyhow::Result<PageRes<UserListRes>> {
    let page = user::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.into();
    Ok(list)
}

pub async fn add(req: UserAddReq, user_: UserPrincipal) -> anyhow::Result<()> {
    req.validate()?;
    let user = UserBuilder::default()
        .id(Some(id::next()))
        .nickname(Some(req.nickname.unwrap_or(req.username.clone())))
        .create_time(Some(tools::now()))
        .build()?;

    let password = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    let user_auth = UserAuthBuilder::default()
        .id(Some(id::next()))
        .user_id(Some(user.id.unwrap()))
        .identity(Some(req.username))
        .secret(Some(password))
        .r#type(Some(IdentityType::Username as i8))
        .create_time(Some(tools::now()))
        .create_user_id(Some(user_.id))
        .build()?;

    let tx = Pool::get()?;
    User::insert(tx, &user).await?;
    UserAuth::insert(tx, &user_auth).await?;

    Ok(())
}

pub async fn delete(req: IdsReq, _user: UserPrincipal) -> anyhow::Result<()> {
    User::delete_by_map(
        Pool::get()?,
        value! {
            "id": req.ids,
        },
    )
    .await?;
    Ok(())
}

pub(crate) async fn update_base_info(
    req: UserUpdateReq,
    _user: UserPrincipal,
) -> anyhow::Result<()> {
    let user = UserBuilder::default()
        .nickname(req.nickname)
        .update_time(Some(tools::now()))
        .build()?;
    User::update_by_map(
        Pool::get()?,
        &user,
        value! {
            "id": req.id,
        },
    )
    .await?;
    Ok(())
}
