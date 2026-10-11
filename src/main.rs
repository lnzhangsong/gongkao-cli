//! gongkao-cli —— 读本 READBOOK 的终端学习形态。
//!
//! 四块内容：行测刷题 / 时评素材 / 常识 / 时政，全部只读主仓 `data/`、记录存本机。
//! 设计依据与接口约定见主仓 `docs/行测终端刷题方案.md`——本仓不改主仓任何代码。

mod app;
mod data;
mod store;
mod ui;
mod views;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gongkao-cli", version, about = "终端刷题与读素材")]
struct Cli {
    /// 题库目录（默认读本主仓 data/）
    #[arg(long, global = true)]
    data: Option<PathBuf>,

    /// 只看统计
    #[arg(long)]
    stats: bool,

    /// 清空作答记录（二次确认）
    #[arg(long)]
    reset: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 行测刷题
    Xingce,
    /// 时评素材
    Read {
        /// 按主题筛选
        #[arg(long)]
        topic: Option<String>,
    },
    /// 常识
    Sense,
    /// 时政
    News,
}

fn default_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GONGKAO_DATA") {
        return PathBuf::from(dir);
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join("Documents/github/gongkao/data");
        if p.is_dir() {
            return p;
        }
    }
    PathBuf::from("data")
}

/// 二次确认：读一行，只认 y/Y，其余一律当取消。EOF（管道输入）也视为取消。
fn confirm_reset() -> Result<bool> {
    use std::io::Write;
    print!("将清空全部作答与阅读记录，不可恢复。确认？(y/N) ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line)? == 0 {
        return Ok(false);
    }
    Ok(matches!(line.trim(), "y" | "Y"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = cli.data.clone().unwrap_or_else(default_data_dir);
    if !root.is_dir() {
        anyhow::bail!(
            "题库目录不存在：{}\n用 --data <path> 或环境变量 GONGKAO_DATA 指定主仓的 data/ 目录",
            root.display()
        );
    }

    if cli.reset {
        // 清空不可逆，要求二次确认（方案 §4.5）。
        if !confirm_reset()? {
            println!("已取消，记录未改动。");
            return Ok(());
        }
        store::reset(&store::records_path()?)?;
        println!("已清空作答记录。");
        return Ok(());
    }

    let mut app = app::App::new(root).context("加载数据失败")?;
    if cli.stats {
        app.print_stats();
        return Ok(());
    }

    if let Some(cmd) = cli.command {
        let screen = match cmd {
            Command::Xingce => app::Screen::XingcePapers,
            Command::Read { topic } => {
                app.topic_filter = topic;
                app::Screen::Articles
            }
            Command::Sense => app::Screen::Sense,
            Command::News => app::Screen::News,
        };
        app.enter(screen);
    }

    ui::run(app)
}
