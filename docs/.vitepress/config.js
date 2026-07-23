export default {
    title: 'Aiway',
    description: 'A high-performance API + AI gateway built with Rust',
    base: '/aiway/',
    themeConfig: {
        logo: '/logo.png',
        siteTitle: false,
        nav: [
            {text: '首页', link: '/'},
            {text: '快速开始', link: '/quick-start'},
            {text: 'GitHub', link: 'https://github.com/xgpxg/aiway'}
        ],
        sidebar: [
            {
                text: '入门',
                items: [
                    {text: '简介', link: '/'},
                    {text: '快速开始', link: '/quick-start'},
                    {text: '架构概览', link: '/architecture'}
                ]
            },
            {
                text: '部署',
                items: [
                    {text: '单机部署', link: '/deployment/standalone'},
                    {text: '集群部署', link: '/deployment/cluster'},
                ]
            },
            {
                text: '网关服务',
                items: [
                    {text: '概述', link: '/gateway/overview'},
                    {text: '请求处理流程', link: '/gateway/request-flow'},
                    {text: '路由', link: '/gateway/routing'},
                    {text: '负载均衡', link: '/gateway/load-balance'},
                    {text: '安全防护', link: '/gateway/security'},
                    {text: '鉴权', link: '/gateway/auth'},
                    {text: '插件系统', link: '/gateway/plugins'},
                    {text: 'AI 模型代理', link: '/gateway/model-proxy'},
                    {text: 'MCP 集成', link: '/gateway/mcp'},
                    {text: '监控与告警', link: '/gateway/monitoring'},
                    {text: '网关扩容', link: '/gateway/scaling'}
                ]
            },
            {
                text: '管理控制台',
                items: [
                    {text: '概述', link: '/console/overview'},
                    {text: '仪表盘', link: '/console/dashboard'},
                    {text: '路由管理', link: '/console/routes'},
                    {text: '服务管理', link: '/console/services'},
                    {text: '插件管理', link: '/console/plugins'},
                    {text: '密钥管理', link: '/console/api-keys'},
                    {text: '防火墙', link: '/console/firewall'},
                    {text: '日志查询', link: '/console/logs'},
                    {text: '域名管理', link: '/console/domains'}
                ]
            },
            {
                text: '接入层',
                items: [
                    {text: '概述', link: '/access/overview'},
                    {text: 'L4 透传', link: '/access/l4-proxy'},
                    {text: 'TLS 终止', link: '/access/tls'}
                ]
            },
            {
                text: '日志服务',
                items: [
                    {text: '概述', link: '/logg/overview'},
                    {text: '索引配置', link: '/logg/index-config'},
                ]
            },
            {
                text: '插件开发',
                items: [
                    {text: '概述', link: '/plugins/overview'},
                    {text: '开发一个插件', link: '/plugins/sdk'},
                    {text: '生命周期与上下文', link: '/plugins/lifecycle'}
                ]
            },
            {
                text: '参考',
                items: [
                    {text: '性能测试', link: '/performance'},
                    {text: 'FAQ', link: '/faq'}
                ]
            }
        ],
        search: {
            provider: 'local',
            options: {
                locales: {
                    zh: {
                        translations: {
                            button: {
                                buttonText: '搜索',
                                buttonAriaLabel: '搜索文档'
                            },
                            modal: {
                                noResultsText: '未找到相关结果',
                                resetButtonTitle: '清除搜索条件',
                                footer: {
                                    selectText: '选择',
                                    navigateText: '切换'
                                }
                            }
                        }
                    }
                }
            }
        },
        socialLinks: [
            {icon: 'github', link: 'https://github.com/xgpxg/aiway'}
        ]
    }
}
