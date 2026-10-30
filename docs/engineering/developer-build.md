# 开发构建与平台验证

本文是三平台环境准备、构建、测试、打包和验证的统一入口。共用编排由 [scripts/desktop.py](../../scripts/desktop.py) 实现；平台文档仅补充部署布局、工具行为和系统限制，不再另维护一套日常命令。

开发构建仍标记实验性。能力和限制见根目录 [README](../../README.md)，支持范围见[支持政策](support-policy.md)。工作包开发与验收完成不代表正式发行获批，许可来源与再分发义务在[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)跟踪。

## 环境与入口

先安装 Python 3.10+、Git、CMake 3.24+、Ninja、rustup 和平台编译器／SDK。Rust 版本读取根目录 rust-toolchain.toml，Qt 固定为 6.8.2，依赖与更新规则见[工具链策略](toolchain.md)。命令均在仓库根目录执行。Windows 可将示例中的 `python3` 改为可调用的 `python`，并在 x64 MSVC 开发环境运行。

| 平台 | 先准备的系统环境 | Qt SDK 子目录 |
| --- | --- | --- |
| macOS | Apple Silicon 原生终端，Xcode Command Line Tools 与有效 macOS SDK | macos |
| Windows | MSVC 2022 x64 C++ 工具与 Windows SDK，x64 Developer PowerShell／命令行 | msvc2022_64 |
| Ubuntu 24.04 | x86_64 C++ 编译器，OpenGL、EGL、xkbcommon 开发包和 X11 运行库 | gcc_64 |

macOS 可用 `xcode-select --install` 准备命令行工具。Ubuntu 基础开发包包含 `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev`；打包还需要 patchelf 与 Debian 包工具，隔离运行验证另需可连接的 Docker。Windows 仅安装 Visual Studio 但未进入开发环境时，cl、link、rc 或 dumpbin 仍可能不可用，预检会指出实际缺失项。共用入口不自动安装系统编译器、SDK 或需要管理员权限的系统包。

Windows 可按 [Community 免费许可路径](windows-licensing.md)准备自有开发工具，阅读并接受该版本条款；只装 Build Tools 或 VC_redist 不单独证明资格。GitHub 标准托管 runner 的 Enterprise 使用及相应再分发另有公开依据，不要求维护者为托管构建购买 Enterprise 或在本机安装 Community IDE，也不将该依据用于自托管机器。现有 2019 工具无需为许可复核而卸载；本地替代配置与当前 2022 发行 SDK 的兼容性须分别验证。

所有命令共用以下参数：

| 参数 | 默认值与用途 |
| --- | --- |
| `--qt-prefix` | 先取环境变量 `XDZIP_QT_PREFIX`，否则使用 `.cache/Qt/6.8.2/<平台子目录>`；应指向 SDK 前缀而不是 bin 目录 |
| `--build-dir` | `build/desktop`，CMake 构建与测试目录 |
| `--output-dir` | `build/package`，交付工件与验证输出目录 |
| `--tools-dir` | `.cache/desktop-tools`，隔离的 Python 工具环境与准备材料 |
| `--configuration` | `Release`，也可显式选择开发所需的构建配置 |

build-dir 与 output-dir 必须使用仓库 build/ 内的独立子目录，以匹配底层打包和验证工具的输出边界。同一次分阶段运行应保持上述参数一致。显式 `--qt-prefix` 优先于环境变量；不需要全局修改 CMAKE_PREFIX_PATH。入口为子进程准备 Qt 与工具搜索环境，普通桌面启动不要沿用离屏测试变量。生成物只写入忽略的 build/、target/、dist/ 或工具缓存，不提交 Git。

## 分阶段执行

各阶段可以单独调用，不隐式补跑前面的准备、构建或测试。缺少前置输入时会报错，先执行相应阶段再重试。每个阶段在开始工作前检查其所需环境，`doctor` 可在不构建或打包的情况下提前查看问题。

`prepare` 核验当前产品格式源码清单。ZIP、TAR/GZIP、7z、RAR、CAB 和光盘容器使用独立 Rust，官方产品源码清单为空，不下载或编译官方 UnRAR、7-Zip 引擎。独立测试参照按各自固定来源准备，与产品运行依赖分开；RAR 有限映射的原始来源通知随打包和对应源码交付保留，见[许可证契约](licensing.md)。`build` 只使用已准备输入，不浮动下载。只测 Rust 时，需要先构建 `xdzip-native-codec`，将 `XDZIP_NATIVE_CODEC` 指向该工作进程，再运行 workspace 测试。原生后端任务闭环另执行 `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`，统一 `test` 与 Foundation CI 均强制执行这部分，忽略状态不能记作格式验证通过。源码、流授权及许可边界见[原生后端契约](native-codecs.md)。

| 命令 | 作用与前置条件 |
| --- | --- |
| `doctor --for-stage build` | 检查 PATH、锁定 Rust、Qt、编译器、SDK 和版本；默认检查 build 所需环境 |
| `prepare` | 准备共用工具和依赖；下载／安装 Qt、Rust 工具链及打包材料须使用下面的显式开关 |
| `build` | 配置并构建真实产品、测试与诊断程序；依赖环境应已就绪，允许源码有未提交改动 |
| `test` | 默认执行 Rust workspace 测试、qmllint 和 CTest；要求已完成同配置构建，允许未提交改动 |
| `package` | 部署运行时、收集许可与来源材料、执行部署审计并生成所选工件；要求干净源码、同提交构建与测试收据及打包工具 |
| `verify` | 核验已有运行时或包，不重新构建，不安装产品、不改默认文件关联；依赖待核验工件与平台验证环境 |

预检报告区分必需缺失、版本不符与当前阶段不需要的工具，不用 Docker 或 Inno Setup 缺失阻止普通开发构建。打包预检会实际检查 Rust/Cargo 调用与锁定依赖元数据，避免到许可收集阶段才发现 PATH 或依赖缓存缺失。失败的命令和验证限制保留在本地／CI 输出中，不把部分完成写成通过。

首次准备固定 Qt 与 Rust 工具链：

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root .cache/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` 使用已安装的 rustup 准备仓库锁定的工具链，不替代 rustup 安装。`--install-qt` 显式下载固定 Qt SDK；已有 SDK 时无需重新下载，直接将其路径交给每个阶段：

```sh
python3 scripts/desktop.py doctor --qt-prefix /path/to/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix /path/to/Qt/6.8.2/gcc_64
```

完成构建后，显式准备完整 `native_backend_tests` 批次使用的公共测试输入。CAB、ISO/UDF 和基础 RAR 夹具位于 `tests/fixtures/cab-iso/` 与 `tests/fixtures/native-archives/`，随项目源码保留；7z 测试使用刚构建的 Rust worker 创建自己的包。这一 runtime 批次不要求官方 7-Zip／UnRAR 工具或挂载映像，Foundation 的其他互操作参照另按[原生后端契约](native-codecs.md)执行。

首次获取固定公共 RAR PPM、旧密码等输入时，显式执行：

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

已有经大小和 SHA-256 核对的固定缓存时，显式采用离线模式，替换缓存路径并保留清单规定的 `junrar/` 子目录布局：

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

准备器的固定来源和通知见 `scripts/rar-rust-fixtures.json` 与其公共兼容清单，输出 `sources.json.environment` 使用源码根目录相对路径。当前对应源码交付不包含这七个公共缓存文件；Cargo 的 `--offline` 不会补齐它们。准备器不会导出 shell 环境，完整 `test` 前按下面的明确交接启动子进程：

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Windows 为 worker 路径加 `.exe`。使用自定义 SDK、build-dir 或 tools-dir 时，将同一组参数附在 worker 路径后，例如 `--qt-prefix /path/to/Qt/6.8.2/gcc_64`，并将 worker 路径调整到实际构建目录。`XDZIP_CAB_BACKEND=rust` 对应固定 Rust CAB 专项；`password` 是公共 fixture 既有的测试值，只交给测试子进程。该交接保留默认 workspace 测试和完整 ignored 批次，不下载输入或隐式构建 worker。

English: Explicitly prepare the fixed public RAR test data, then pass sources.json.environment, the freshly built worker and the existing public test value to the test child. CAB/optical/basic RAR fixtures ship in the project source; 7z creates its own inputs. The seven public cache files are separate prerequisites. A verified cache supports --archives/--offline; initial fixed downloads are a separate explicit step. Test stages do not implicitly prepare or download these fixture inputs, and the complete ignored runtime selection is retained.

test 仅对子进程启用 offscreen／software，不污染后续命令的桌面环境。日常修改只需按需要重新执行 build 和 test。只验证 Rust 核心仍可执行 `cargo test --workspace --locked`，不需要 Qt。`test --skip-rust` 只用于已经由其他检查负责 Rust 的编排，例如 Foundation probes 已执行同提交 Rust 门槛的 Qt CI，不表示跳过 Qt 测试。

浏览器测试可直接运行或由现有套件入口分组执行。每个套件进程在自身 CMake 构建目录的 `test-runtime/` 内创建独立临时根，退出时在测试对象销毁后、Qt 应用销毁前恢复 TEMP、TMP、TMPDIR 并清理；默认设置文件同样隔离。同组辅助进程继承父临时根，保持单实例 IPC 的目录身份，使用私有前置探针并在父进程清理前结束；不同分组不共享。语言重启及打开方式子进程显式传入的测试配置继续共享，不改用新的空配置。运行前实际验证子文件创建、锁、暂存、原子提交与设置读取；失败以 `Fixture precondition` 和非零退出码报告，不跳过测试或放宽窗口隐藏等断言。目录存在及 QTemporaryDir::isValid 不单独证明可写，也不通过调整系统权限使夹具通过。

测试中的配置路径使用 `tests/test_environment` 作用域保存和恢复，Windows 保持宽字符值，并在系统环境块中区分未设置和显式空值；Unix 保留原环境字节。MSVC CRT 的设置 API 会删除空值，原生环境块另行恢复，因此空值的存在性使用 helper 的 processValue 或 QProcessEnvironment 检查，不以 Qt 的 CRT 空值／null 区别作保证。该 helper 仅限串行测试主线程，在相关引擎／工作线程创建前设置，在销毁／join 后恢复，不用于产品代码或并发环境修改。ASCII、中文及 ANSI 不可表示字符须核对原路径实际文件、语言恢复子进程和失败时窗口／草稿保留。自动化结果仍与真人设置操作分开记录。

English: Browser suite processes use unique temporary roots under their CMake build directory; auxiliary children retain the parent's temp identity for IPC, with private preflight probes and teardown before parent cleanup. Different shards remain isolated. Restore variables and clean fixtures after test teardown, before Qt application destruction. Probes cover child writes, locking, staging, atomic commit and settings readback, and fixture failures remain nonzero failures. Scoped overrides preserve Unicode paths and absent versus empty values in the native environment; MSVC's CRT setter deletes empty entries, so use processValue or QProcessEnvironment for empty presence. Mutate overrides only on the serial fixture main thread, before related engines/workers and after teardown/join. Explicit settings restart children retain their shared profile. These checks do not replace native manual acceptance or change product configuration semantics.

真实产品在所有平台统一使用 `xdzip` 名称。默认构建后的入口为：

| 平台 | 入口 |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

组件示例 `xdzip-ui-preview` 不用于真实文件验收。更换 build-dir 后相应调整路径；版本、渠道、提交与 dirty 状态由实际构建身份记录。应用模块与低层 CMake 调试见[Qt 开发应用](../../apps/desktop/README.md)。

## 打包与验证

test 与 package 会检查构建收据、源码指纹、配置、SDK 身份及实际产品文件；修改源码或切换配置后先重新 build。打包前提交待交付源码，再从该干净提交构建和测试。package 不通过忽略脏状态、修改产物身份或沿用旧报告来绕过检查。许可材料、Rust 元数据与实际 CLI 版本必须对应这次源码。已有打包目录的使用与覆盖限制由各平台底层工具检查，重建候选时优先使用新的 output-dir，避免混入旧文件。

Windows／Linux 先准备打包工具，再执行统一阶段：

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` 准备隔离 Python 校验依赖、锁定许可正文和 schema 缓存，以及平台打包材料；Windows 包括固定 CRT 和 Inno Setup。系统工具仍由开发者预先安装。`package --kind runtime` 只准备部署运行目录及其审计材料；随后可用同一 output-dir 执行 `package --kind installer`，从已核验运行目录生成安装器，Linux 同时生成保留执行权限的 tar.gz。`--kind all` 为默认值，在一次命令内依次完成这两部分。Windows 打包使用 Release 或 RelWithDebInfo，不部署 Debug Qt 库；Windows 提供部署目录和当前用户安装器，Linux 提供运行目录、tar.gz 和 .deb。Windows 核验需要同配置的浏览测试程序和 Qt Test／offscreen 测试输入，Linux 核验需要构建测试生成的 m2-flat.zip；测试文件不会进入产品目录。verify 的 `--kind runtime` 核验运行时，默认 `--kind all` 核验已选交付工件；普通核验不执行安装、升级／回退或用户文件关联变更。

macOS DMG 须单独选择允许注册文档候选的主构建，避免普通开发／测试副本进入“打开方式”：

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

普通 build 的 macOS handler rank 默认为 `None`，不更改用户默认关联；仅长期保留的主应用或 DMG 输入选择 `Alternate`。macOS 本地 package 只支持 `all`，生成 arm64 开发 DMG 并执行必要 ad-hoc 完整性签名，verify 使用重定位后的 Cocoa／Metal 及真实解压检查。CI 禁止运行 Mac package，不部署、签名、压缩或上传应用包。完整限制与副本注册维护见[Mac 部署指南](macos-deployment.md)。

部署布局和底层工具分别见[Linux 部署](linux-deployment.md)、[Mac DMG](macos-deployment.md)及[安装与文件关联](installers-associations.md)。Windows/Linux 安装器未正式签名，macOS 未做 Developer ID 签名或公证。安装和运行环境问题使用实际产品与对应平台验证，不以 SDK 下可运行代替独立包核验。

## 与 CI 的关系

[Qt 工作流](../../.github/workflows/qt-components.yml)通过同一 desktop.py 执行准备、构建、基础测试以及 Windows/Linux 打包和运行时核验。[ci-desktop.ps1](../../scripts/ci-desktop.ps1)保留 CI 专用的重复并发生命周期、性能、视觉、多 DPR、互操作、来源证据及历史安装兼容扩展，不作为开发机日常构建入口。共用阶段不要求 RUNNER_TEMP 或 RUNNER_ARCH；CI 显式传入其路径，普通本地命令使用上述默认目录。

历史完整产品、真实安装升级／回退及关联生命周期只在一次性 Windows/Linux CI 主机或隔离容器执行，不因统一入口就开放为普通本地 verify 的默认行为。macOS CI 继续使用 SDK 模式部署审计和 Cocoa 启动／文件打开检查，仅保存日志和 JSON。完整门槛与证据边界见[CI 策略](ci-policy.md)。

## 文件与任务边界

打开只浏览，不自动解压。普通同名文件支持询问、跳过、改名及明确替换；内容校验和同步后才整文件发布。提交前取消或写入失败保留旧文件，早前已提交的解压项继续保留。关闭任务窗口仅隐藏详情，退出由活动任务决策和异步 shutdown 屏障处理。

ZIP 在受限 POSIX 条件下可恢复相对符号链接和可执行位，不恢复特权位、扩展属性或原始修改时间，不代表应用包安装或签名验证。来源与目标必须满足[安全解压契约](m3-extraction.md)。崩溃暂存只提示并保留供检查，不自动恢复或删除未知内容。

## 证据与复验

按[通用验证方法](../qa/README.md)和[桌面清单](../qa/private-test-manual.md)选择受影响项。构建与测试收据分别保存在 build-dir 下的 `desktop-build.json`、`desktop-test.json`，prepare、package 和 verify 回执位于 output-dir 的 `<stage>.json`，并关联底层验证证据。自动化收据记录实际阶段与来源；离屏／软件渲染、原生 Cocoa 检查和人工专项分别登记，不相互替代。既有验收在 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)保留，不因重新生成候选清单而重做。

Windows/Linux 工件交付同提交源码和许可通知，Actions 保存 14 天；过期后按身份和锁定输入重建。deployment-audit.json、源码及文件级 CycloneDX 清单用于来源核验，覆盖仍可能为 incomplete，不能当作许可批准。源码重建与 Qt 替换方法见[源码交付](source-delivery.md)及[自动验证契约](wp12-automation.md)。

所有构建、截图、量测、报告与包只保留为忽略的本地或 CI 工件。不要把生成的 pending 表、离屏检查或部分 SBOM 写成干净环境与全部发行门槛通过。
