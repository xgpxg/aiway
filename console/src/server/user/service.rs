use crate::cache::caches::CacheKey;
use crate::server::auth::UserPrincipal;
use crate::server::db::models::user::User;
use crate::server::db::models::user_auth::{IdentityType, UserAuth, UserAuthBuilder};
use crate::server::db::{Pool, tools};
use crate::server::user::request::{LoginReq, UpdatePasswordReq};
use crate::server::user::response::{LoginRes, OtherInfo, UserBaseInfo, UserCenterRes};
use anyhow::bail;
use rbs::value;
use std::time::Duration;

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
            let user_id = user_auth
                .into_iter()
                .find(|auth| {
                    bcrypt::verify(&req.secret, &auth.secret.clone().unwrap_or_default())
                        .unwrap_or(false)
                })
                .map(|auth| auth.user_id.unwrap())
                .ok_or_else(|| anyhow::anyhow!("用户名或密码不正确"))?;
            user_id
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
