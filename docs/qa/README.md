# 验证方法与证据管理

本目录保存可重复使用的验收要求、测试方法、样本规则、审查流程和空白模板。它不保存具体版本的验收结果、阶段报告、截图、测量 JSON、运行日志或交接进度。

- [产品验收要求](acceptance.md)
- [样本与夹具规则](fixtures.md)
- [原生桌面与安装人工验证方法](private-test-manual.md)
- [发行审查流程](release-review.md)
- [CI 策略](../engineering/ci-policy.md)
- [部署依赖与 SBOM 契约](../engineering/deployment-audit.md)

## 状态与证据放在哪里

具体工作、验收结论、未覆盖项和阻塞原因记录在 [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues)。工作包、缺陷与发行许可复核分开跟踪；同一问题复用既有 issue，不用新文档累积“第几批完成”。WP-12 历史验收保留于 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)，发行许可复核见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)，不因目录整理重复要求用户验收。

截图、日志、逐次测量和机器报告输出到 `build/` 或 `dist/`，按 CI 保留政策上传 [Actions 工件](https://github.com/xiaodaozhi/xdzip/actions)。issue 引用实际运行、提交、工件摘要和限制，不粘贴整个生成报告，不将过期工件或配置成功冒充验证通过。长期发行材料由实际 Release 随包提供，发布仍需明确授权。

`design/hifi/` 保存设计原稿，`design/assets/` 保存产品素材，`tests/fixtures/` 保存自有可再分发回归输入；它们是持久输入，与实际运行的截图或测量输出不同。内容边界和历史整理方法见[仓库内容管理](../engineering/repository-hygiene.md)。
