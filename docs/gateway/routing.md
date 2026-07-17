# 路由

路由是aiway的核心功能，当一个请求到来时，aiway会根据配置的路由规则，将请求转发给对应的服务处理。

支持以下路由规则：

## Host匹配

  支精确域名（ 如：www.example.com ）和泛域名匹配（ 如：*.example.com ）。泛域名仅支持单个通配符。

## Method匹配

  使用HTTP的请求方法进行匹配，如GET、POST、PUT、DELETE、PATCH等。

## Path匹配

  使用HTTP的请求路径部分进行匹配。支持统配符，如：`/api/*/some`、`/api/**`

## Header匹配

  使用HTTP请求的Header进行匹配。

## Query匹配

  使用HTTP请求的Query参数进行匹配。

