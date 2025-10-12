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
    name           varchar(100)  not null,                   -- 路由名称
    description    varchar(500),                             -- 路由描述
    status         varchar(20)   not null default 'Disable', -- 状态：Disable | Ok
    host           varchar(100),                             -- 需要匹配的域名
    -- prefix         varchar(100),                             -- 路由前缀
    path           varchar(500)  not null,                   -- 路由路径
    -- strip_prefix   tinyint(1)    not null default 1,         -- 是否去除路径前缀
    service        varchar(100)  not null,                   -- 目标服务名
    header         varchar(1000) not null,                   -- 按请求头匹配
    query          varchar(1000) not null,                   -- 按请求参数匹配
    pre_filters    varchar(500)  not null,                   -- 请求阶段过滤器，JSON数组
    post_filters   varchar(500)  not null,                   -- 响应阶段过滤器，JSON数组
    create_user_id bigint,                                   -- 创建人ID
    update_user_id bigint,                                   -- 修改人ID
    create_time    datetime,                                 -- 创建时间
    update_time    datetime,                                 -- 更新时间
    remark         varchar(500),                             -- 备注
    is_delete      tinyint(1)    not null default 0          -- 是否删除
);
create table if not exists service
(
    id             bigint primary key,
    name           varchar(100)  not null,                   -- 服务名称，全局唯一
    description    varchar(500)  not null,                   -- 服务描述。注意这个描述要求非空，用于在控制台展示
    status         varchar(20)   not null default 'Disable', -- 状态：Disable | Ok
    nodes          varchar(5000) not null,                   -- 服务节点，JSON数组，支持IP和域名，如["http://127.0.0.1:8080"]
    lb             varchar(20)   not null,                   -- 负载均衡策略：random | round_robin
    create_user_id bigint,                                   -- 创建人ID
    update_user_id bigint,                                   -- 修改人ID
    create_time    datetime,                                 -- 创建时间
    update_time    datetime,                                 -- 更新时间
    remark         varchar(500),                             -- 备注
    is_delete      tinyint(1)    not null default 0          -- 是否删除
);

create table if not exists plugin
(
    id             bigint primary key,
    name           varchar(100) not null,          -- 插件名称
    description    varchar(500),                   -- 插件描述
    url            varchar(500) not null,          -- 下载地址，该地址用于gateway下载插件，需保证从gateway处可以访问。
    version        varchar(50)  not null,          -- 插件版本，格式为0.1.0
    default_config text,                           -- 插件默认配置，JSON字符串
    create_user_id bigint,                         -- 创建人ID
    update_user_id bigint,                         -- 修改人ID
    create_time    datetime,                       -- 创建时间
    update_time    datetime,                       -- 更新时间
    remark         varchar(500),                   -- 备注
    is_delete      tinyint(1)   not null default 0 -- 是否删除
);

create table if not exists api_key
(
    id             bigint primary key,
    name           varchar(100) not null,              -- 密钥名称
    principal      varchar(500),                       -- 密钥所属的主体标识，可以为空
    secret         varchar(100) not null,              -- 密钥
    status         varchar(20)  not null default 'Ok', -- 状态：Disable | Ok
    eff_time       datetime     not null,              -- 生效时间，默认当前时间
    exp_time       datetime,                           -- 失效时间，为空表示永久有效
    create_user_id bigint,                             -- 创建人ID
    update_user_id bigint,                             -- 修改人ID
    create_time    datetime,                           -- 创建时间
    update_time    datetime,                           -- 更新时间
    remark         varchar(500),                       -- 备注
    is_delete      tinyint(1)   not null default 0     -- 是否删除
);


-- 网关节点
create table if not exists gateway_node
(
    id                  bigint primary key,
    node_id             varchar(100) not null,          -- 节点ID，md5(ip:port)后取前8位
    node_name           varchar(100),                   -- 节点名称
    ip                  varchar(100) not null,          -- IP
    port                int          not null,          -- 端口
    status              varchar(50)  not null,          -- 节点状态：Online | Offline | Unknown
    status_msg          varchar(500),                   -- 节点状态信息
    last_heartbeat_time datetime,                       -- 最后一次心跳时间
    create_user_id      bigint,                         -- 创建人ID
    update_user_id      bigint,                         -- 修改人ID
    create_time         datetime,                       -- 创建时间
    update_time         datetime,                       -- 更新时间
    remark              varchar(500),                   -- 备注
    is_delete           tinyint(1)   not null default 0 -- 是否删除
);

-- 网关节点状态
create table if not exists gateway_node_state
(
    id                             bigint primary key,
    node_id                        varchar(100) not null,           -- 节点ID
    ts                             bigint       not null,           -- 毫秒时间戳
    os                             varchar(50),                     -- 操作系统及版本，如: Ubuntu 22.04
    host_name                      varchar(100),                    -- 主机名
    cpu_usage                      float        not null default 0, -- cpu 使用率
    mem_total                      bigint       not null default 0, -- 内存状态 - 总内存，单位：Bytes
    mem_free                       bigint       not null default 0, -- 内存状态 - 空闲内存，单位：Bytes
    mem_used                       bigint       not null default 0, -- 内存状态 - 使用内存，单位：Bytes
    disk_total                     bigint       not null default 0, -- 磁盘状态 - 总空间，单位：Bytes
    disk_free                      bigint       not null default 0, -- 磁盘状态 - 空闲空间，单位：Bytes
    net_rx                         bigint       not null default 0, -- 网络状态 - 接收的字节数
    net_tx                         bigint       not null default 0, -- 网络状态 - 发送的字节数
    net_tcp_conn_count             bigint       not null default 0, -- 网络状态 - TCP连接数
    avg_qps                        bigint       not null default 0, -- 平均QPS
    interval_request_count         bigint       not null default 0, -- 区间内请求数
    interval_request_invalid_count bigint       not null default 0, -- 区间内无效请求数
    interval_response_2xx_count    bigint       not null default 0, -- 区间内2xx响应数
    interval_response_3xx_count    bigint       not null default 0, -- 区间内3xx响应数
    interval_response_4xx_count    bigint       not null default 0, -- 区间内4xx响应数
    interval_response_5xx_count    bigint       not null default 0, -- 区间内5xx响应数
    interval_http_connect_count    bigint       not null default 0, -- 区间内http连接数
    interval_avg_response_time     bigint       not null default 0, -- 区间内平均响应时间
    request_count                  bigint       not null default 0, -- 累计请求数
    request_invalid_count          bigint       not null default 0, -- 累计无效请求数
    response_2xx_count             bigint       not null default 0, -- 累计2xx响应数
    response_3xx_count             bigint       not null default 0, -- 累计3xx响应数
    response_4xx_count             bigint       not null default 0, -- 累计4xx响应数
    response_5xx_count             bigint       not null default 0, -- 累计5xx响应数
    http_connect_count             bigint       not null default 0, -- 累计http连接数
    avg_response_time              bigint       not null default 0, -- 累计平均响应时间
    create_time                    datetime                         -- 创建时间
);
create index if not exists idx_node_id on gateway_node_state (node_id);
create index if not exists idx_ts on gateway_node_state (ts);


create table if not exists message
(
    id          bigint primary key,
    type        varchar(50) not null,           -- 消息类型：system | alert
    level       varchar(50) not null,           -- 消息级别：info | warn | error
    title       varchar(500),                   -- 标题
    content     text        not null,           --  内容
    read_status tinyint(1)  not null default 0, -- 0未读 1已读
    create_time datetime    not null
);
-- -------------------------------- 初始化用户 --------------------------------------
insert or ignore into user(id, nickname)
values (1, 'admin');
insert or ignore into user_auth(id, user_id, type, identity, secret)
values (1, 1, 1, 'admin', '$2b$12$uMYLbc5X3VIPkBxBKa7w9OrLwQEzyhCZe8.aGVxtQmpqCx4okFMoW');

