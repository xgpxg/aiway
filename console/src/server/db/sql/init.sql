create table if not exists system_config
(
    config_key   varchar(100) not null primary key,
    config_value text         null
);

create table if not exists user
(
    id          bigint primary key,
    nickname    varchar(500) not null,          -- 昵称
    avatar      varchar(500),                   -- 头像
    status      tinyint(1),                     -- 状态：0禁用 1正常
    create_time datetime,                       -- 创建时间
    update_time datetime,                       -- 更新时间
    remark      varchar(500),                   -- 备注
    is_delete   tinyint(1)   not null default 0 -- 是否删除
);

create table if not exists user_auth
(
    id             bigint               not null primary key,
    user_id        bigint               not null, -- 用户ID
    type           tinyint(1)           not null, -- 认证类型: 1用户名密码 2邮箱
    identity       varchar(500)         not null, -- 认证标识
    secret         varchar(500)         null,     -- 认证密钥
    create_user_id bigint               null,     -- 创建人ID
    update_user_id bigint               null,     -- 修改人ID
    create_time    datetime             null,     -- 创建时间
    update_time    datetime             null,     -- 更新时间
    remark         varchar(500)         null,     -- 备注
    is_delete      tinyint(1) default 0 null      -- 是否删除
);


create table if not exists route
(
    id             bigint primary key,
    name           varchar(100) not null,           -- 路由名称
    description    varchar(500),                    -- 路由描述
    host           varchar(100),                    -- 需要匹配的域名
    prefix         varchar(100),                    -- 路由前缀
    path           varchar(500) not null,           -- 路由路径
    strip_prefix   tinyint(1)   not null default 1, -- 是否去除路径前缀
    service        varchar(100) not null,           -- 目标服务名称
    protocol       varchar(10)  not null,           -- 请求协议：http | https
    method         varchar(10)  not null,           -- 请求方法：GET | POST | PUT | DELETE | HEAD | OPTIONS | PATCH | TRACE | CONNECT
    header         varchar(1000),                   -- 按请求头匹配
    query          varchar(1000),                   -- 按请求参数匹配
    pre_filters    varchar(500),                    -- 请求阶段过滤器，JSON数组
    post_filters   varchar(500),                    -- 响应阶段过滤器，JSON数组
    create_user_id bigint,                          -- 创建人ID
    update_user_id bigint,                          -- 修改人ID
    create_time    datetime,                        -- 创建时间
    update_time    datetime,                        -- 更新时间
    remark         varchar(500),                    -- 备注
    is_delete      tinyint(1)   not null default 0  -- 是否删除
);
create table if not exists service
(
    id             bigint primary key,
    name           varchar(100) not null,          -- 服务名称
    description    varchar(500),                   -- 服务描述
    nodes          varchar(100) not null,          -- 节点地址，JSON数组，支持IP和域名，如["http://127.0.0.1:8080"]
    load_balance   varchar(20)  not null,          -- 负载均衡算法：random | round_robin
    create_user_id bigint,                         -- 创建人ID
    update_user_id bigint,                         -- 修改人ID
    create_time    datetime,                       -- 创建时间
    update_time    datetime,                       -- 更新时间
    remark         varchar(500),                   -- 备注
    is_delete      tinyint(1)   not null default 0 -- 是否删除
);

-- -------------------------------- 初始化用户 --------------------------------------
insert or ignore into user(id, nickname)
values (1, 'admin');
insert or ignore into user_auth(id, user_id, type, identity, secret)
values (1, 1, 1, 'admin', '$2b$12$uMYLbc5X3VIPkBxBKa7w9OrLwQEzyhCZe8.aGVxtQmpqCx4okFMoW');