use std::time::Duration;

use anstyle::{AnsiColor, Style};
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand, builder::styling::Styles};
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{Context, IntoDiagnostic, Result, miette};
use tokio::{fs, io};

const SUCCESS: Style = AnsiColor::Green.on_default().bold();
const DIM: Style = AnsiColor::White.on_default().dimmed();

const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::White.on_default().dimmed());

#[derive(Parser)]
#[command(name = "Yandex Blog", version = "0.0.1", styles = CLAP_STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// whether to use grpc
    #[arg(short, long)]
    grpc: bool,
    /// an alt address
    #[arg(short = 'H', long)]
    http: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: i64,
    },
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: i64,
    },
    List {
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
}

struct TokenStorage(String);

const TOKEN_PATH: &str = "./.blog-token";

impl TokenStorage {
    async fn from_file() -> io::Result<Self> {
        Ok(Self(fs::read_to_string(TOKEN_PATH).await?))
    }

    async fn into_file(self) -> io::Result<()> {
        fs::write(TOKEN_PATH, self.0).await
    }

    fn new(t: String) -> Self {
        Self(t)
    }
}

impl From<TokenStorage> for String {
    fn from(t: TokenStorage) -> Self {
        t.0
    }
}

fn fmt_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn base_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn posts_table(posts: &[blog_proto::Post]) -> Table {
    let mut table = base_table();
    table.set_header(vec!["ID", "Title", "Author", "Created", "Updated"]);
    for p in posts {
        table.add_row(vec![
            Cell::new(p.id),
            Cell::new(&p.title),
            Cell::new(p.author_id),
            Cell::new(fmt_ts(p.created_at)),
            Cell::new(fmt_ts(p.updated_at)),
        ]);
    }
    table
}

fn post_table(p: &blog_proto::Post) -> Table {
    let mut table = base_table();
    table.set_header(vec!["Field", "Value"]);
    table
        .add_row(vec![Cell::new("ID"), Cell::new(p.id)])
        .add_row(vec![Cell::new("Title"), Cell::new(&p.title)])
        .add_row(vec![Cell::new("Author"), Cell::new(p.author_id)])
        .add_row(vec![Cell::new("Created"), Cell::new(fmt_ts(p.created_at))])
        .add_row(vec![Cell::new("Updated"), Cell::new(fmt_ts(p.updated_at))])
        .add_row(vec![Cell::new("Content"), Cell::new(&p.content)]);
    table
}

fn spinner(msg: &'static str) -> ProgressBar {
    let s = ProgressBar::new_spinner();
    s.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} [{elapsed}]")
            .expect("static template")
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    s.set_message(msg);
    s.enable_steady_tick(Duration::from_millis(80));
    s
}

async fn load_token() -> Result<TokenStorage> {
    TokenStorage::from_file()
        .await
        .into_diagnostic()
        .wrap_err("no stored token")
        .with_context(|| format!("run `login` first (looked in {TOKEN_PATH})"))
}

#[tokio::main]
async fn main() -> Result<()> {
    use Commands::*;
    let cli = Cli::parse();
    let transport = if cli.grpc {
        Transport::Grpc
    } else {
        Transport::Http(cli.http)
    };

    let mut client = BlogClient::new(transport)
        .await
        .into_diagnostic()
        .wrap_err("failed to construct client")?;

    match cli.command {
        Register {
            username,
            email,
            password,
        } => {
            let s = spinner("Signing you up...");
            let r = client.register(username, email, password).await;
            s.finish_and_clear();
            let usertoken = r.into_diagnostic().wrap_err("signup failed")?;
            let username = usertoken
                .user
                .ok_or_else(|| miette!("server response missing user"))?
                .username;
            TokenStorage::new(usertoken.token)
                .into_file()
                .await
                .into_diagnostic()
                .wrap_err("could not persist token")?;
            anstream::println!(
                "{SUCCESS}Welcome on board, {username}{SUCCESS:#}"
            );
        }
        Login { username, password } => {
            let s = spinner("Logging in...");
            let r = client.login(username, password).await;
            s.finish_and_clear();
            let r = r.into_diagnostic().wrap_err("login failed")?;
            TokenStorage::new(r.token)
                .into_file()
                .await
                .into_diagnostic()
                .wrap_err("could not persist token")?;
            anstream::println!("{SUCCESS}Logged in{SUCCESS:#}");
        }
        Create { title, content } => {
            client.set_token(load_token().await?);
            let s = spinner("Creating post...");
            let r = client.create_post(title, content).await;
            s.finish_and_clear();
            r.into_diagnostic().wrap_err("create failed")?;
            anstream::println!("{SUCCESS}Post created{SUCCESS:#}");
        }
        Get { id } => {
            let s = spinner("Fetching post...");
            let r = client.get_post(id).await;
            s.finish_and_clear();
            let p = r
                .into_diagnostic()
                .wrap_err_with(|| format!("could not fetch post {id}"))?;
            anstream::println!("{}", post_table(&p));
        }
        Update { id, title, content } => {
            client.set_token(load_token().await?);
            let s = spinner("Updating post...");
            let r = async {
                let p = client.get_post(id).await?;
                client
                    .update_post(
                        id,
                        &title.unwrap_or(p.title),
                        &content.unwrap_or(p.content),
                    )
                    .await
            }
            .await;
            s.finish_and_clear();
            r.into_diagnostic()
                .wrap_err_with(|| format!("update of post {id} failed"))?;
            anstream::println!("{SUCCESS}Post {id} updated{SUCCESS:#}");
        }
        Delete { id } => {
            client.set_token(load_token().await?);
            let s = spinner("Deleting post...");
            let r = client.delete_post(id).await;
            s.finish_and_clear();
            r.into_diagnostic()
                .wrap_err_with(|| format!("delete of post {id} failed"))?;
            anstream::println!("{SUCCESS}Post {id} deleted{SUCCESS:#}");
        }
        List { limit, offset } => {
            let s = spinner("Fetching posts...");
            let r = client
                .list_posts(
                    limit.and_then(|l| l.try_into().ok()),
                    offset.and_then(|o| o.try_into().ok()),
                )
                .await;
            s.finish_and_clear();
            let r = r.into_diagnostic().wrap_err("list failed")?;
            if r.posts.is_empty() {
                anstream::println!("{DIM}No posts here yet{DIM:#}");
            } else {
                anstream::println!("{}", posts_table(&r.posts));
            }
        }
    }

    Ok(())
}
