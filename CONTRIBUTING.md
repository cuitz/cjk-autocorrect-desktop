# Contributing to CJK AutoCorrect Desktop

感谢你对 CJK AutoCorrect Desktop 的关注！欢迎贡献代码、报告问题或提出建议。

## 报告问题

- 使用 [GitHub Issues](../../issues) 提交 Bug 报告或功能建议
- 请提供详细的复现步骤、系统环境和预期行为

## 开发流程

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交改动：`git commit -m 'feat: add your feature'`
4. 推送分支：`git push origin feature/your-feature`
5. 提交 Pull Request

## 开发环境

```bash
# 安装依赖
pnpm install

# 启动开发模式
pnpm tauri dev
```

前置要求详见 [README](./README.md#-快速开始)。

## 代码规范

- **Rust**：遵循 `cargo fmt` 和 `cargo clippy` 建议
- **TypeScript/React**：遵循现有代码风格，组件使用函数式组件 + Hooks
- **提交信息**：使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式

## 许可证

提交的代码将按 [MIT License](./LICENSE) 发布。
