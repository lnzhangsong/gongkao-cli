# 读本 READBOOK · 终端学习（gongkao-cli）

把行测刷题、时评素材、常识、时政搬进终端：Rust + [ratatui](https://ratatui.rs) 做 TUI，
逐题作答即时判分、翻两段时评、瞟两条常识——不离开终端。

**独立仓库**：数据只读主仓 [gongkao](https://github.com/) 的 `data/`，本仓不改主仓任何代码。
设计依据与接口约定见主仓 `docs/行测终端刷题方案.md`。

## 内容

| 板块     | 数据源                                        | 形态                              | 状态     |
| -------- | --------------------------------------------- | --------------------------------- | -------- |
| 行测刷题 | 主仓 `data/xingce/*.json`（44 套）             | 选卷 → 逐题作答 → 判分 → 解析      | ✅ 可用   |
| 时评素材 | 主仓 `data/articles.db` 的 `articles`（517 篇）| 列表 → 正文分段翻页               | ✅ 可用   |
| 常识     | 本仓 `data/common-sense.json`（40 条）         | 按主题分组 → 词条详情             | ✅ 可用   |
| 时政     | 本仓 `data/politics-hot.json`（20 条）         | 按主题分组 → 词条详情             | ✅ 可用   |

原方案只做行测刷题；2026-10-11 用户扩为「终端学习」，加了时评与常识/时政两块。
常识/时政词条为**手写、质量优先**（首批量少，扩量待做）。

## 准备

1. **Rust 工具链**（本机已装 1.98+）。
2. **主仓数据就绪**——时评读的是构建产物 `data/articles.db`（未入 git），先在主仓跑一次：

   ```bash
   cd ~/Documents/github/gongkao && vp run db:rebuild
   ```

   行测题库 `data/xingce/*.json` 是源文件，开箱即有。

## 构建与运行

```bash
cargo build --release          # 产物 target/release/gongkao-cli
cargo run                      # 调试运行，进 TUI 主菜单
```

默认从 `~/Documents/github/gongkao/data` 读数据；换路径用 `--data` 或环境变量 `GONGKAO_DATA`。

```bash
cargo run -- --data /path/to/gongkao/data
GONGKAO_DATA=/path/to/data cargo run
```

## 命令

```bash
gongkao-cli                    # 进 TUI 主菜单
gongkao-cli xingce             # 直达行测刷题
gongkao-cli read [--topic 文化自信]   # 直达时评
gongkao-cli sense              # 常识
gongkao-cli news               # 时政
gongkao-cli --stats            # 只看统计（不进 TUI）
gongkao-cli --reset            # 清空作答记录
gongkao-cli --data <path>      # 指定题库目录
```

## 快捷键

| 界面     | 键                                                    |
| -------- | ----------------------------------------------------- |
| 通用     | `q` 退出（详情页返回上一层）                          |
| 列表     | `↑`/`↓` 选，`回车` 进，`esc` 返回                     |
| 行测作答 | `1`–`4` 选选项；判分后 `回车` 看解析 → `回车` 下一题 |
| 时评正文 | `j`/`k` 逐段，`space`/`b` 翻页，`esc` 返回列表        |
| 常识/时政| `↑`/`↓` 选，`回车` 看详情，`esc` 返回                 |

## 记录

作答与阅读记录落盘在 `~/.local/share/gongkao-cli/records.json`（`--reset` 清空）。
**与 Web 端、云端不互通**——裁定「先不连」（主仓方案 §二）。作答记录键形 `paperId#qIdx`
与 Web 端 `xg_answers` 一致，将来若要互通可平移。

## 已知边界

- **含图题跳过**：图形推理、资料分析材料是图片，终端显不了。可刷判定 = 有答案 + 无图 +
  题组有文本材料；进入前会提示跳过几题，不静默漏题。
- **常识/时政量少**：首批 40 + 20 条，远未覆盖国考常识考点——只作随手瞟，**不是备考主力**。
  扩量照 `data/common-sense.json` 的字段补即可（`term/group/meaning/distinction/example`）。
- 不做 AI 讲解、笔记、错题本跨卷汇总（Web 端已有）。

## 开发

```bash
cargo test      # 数据层判定逻辑（跳过判定 / content_json 解析 / 记录往返）
cargo clippy    # 无告警（serde 结构体的未读字段除外）
```

## 许可

源码 AGPL-3.0。题目、时评、词条等内容来自主仓 `data/`，**不在授权范围内**，版权归各自
权利人，仅随仓库供个人学习——本工具**只自用、不分发**。
