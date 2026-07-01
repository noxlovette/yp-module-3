use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use tokio::{fs, io};

#[derive(clap::Parser)]
#[command(name = "Yandex Blog")]
#[command(version = "0.0.1")]
struct Cli {
    /// the command to fire
    #[command(subcommand)]
    command: Commands,
    /// whether to use grpc
    #[arg(short, long)]
    grpc: bool,
    /// an alt address
    #[arg(short, long)]
    http: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Register {
        username: String,
        email: String,
        password: String,
    },
    Login {
        username: String,
        password: String,
    },
    Create {
        title: String,
        content: String,
    },
    Get {
        id: i64,
    },
    Update {
        id: i64,
        title: Option<String>,
        content: Option<String>,
    },
    Delete {
        id: i64,
    },
    List {
        limit: Option<i64>,
        offset: Option<i64>,
    },
}

struct TokenStorage(String);

const TOKEN_PATH: &'static str = "./blog-token";

impl TokenStorage {
    async fn from_file() -> io::Result<Self> {
        Ok(Self(fs::read_to_string(TOKEN_PATH).await?))
    }

    async fn into_file(self) -> io::Result<()> {
        Ok(fs::write(TOKEN_PATH, self.0).await?)
    }

    fn new(t: String) -> Self {
        Self(t)
    }
}

impl Into<String> for TokenStorage {
    fn into(self) -> String {
        self.0
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use Commands::*;
    let cli = Cli::parse();
    let transport = if cli.grpc {
        Transport::Grpc
    } else {
        Transport::Http(cli.http)
    };

    let mut client = BlogClient::new(transport).await?;

    match cli.command {
        Register {
            username,
            email,
            password,
        } => {
            let r = client.register(username, email, password).await?;
            TokenStorage::new(r.token).into_file().await?;
        }
        Login { username, password } => {
            let r = client.login(username, password).await?;
            TokenStorage::new(r.token).into_file().await?;
        }
        Create { title, content } => {
            client.set_token(TokenStorage::from_file().await?);
            client.create_post(title, content).await?;
        }
        Get { id } => {
            client.get_post(id).await?;
        }
        Update { id, title, content } => {
            client.set_token(TokenStorage::from_file().await?);
            let p = client.get_post(id).await?;
            client
                .update_post(
                    id,
                    &title.unwrap_or(p.title),
                    &content.unwrap_or(p.content),
                )
                .await?;
        }
        Delete { id } => {
            client.set_token(TokenStorage::from_file().await?);
            client.delete_post(id).await?;
        }
        List { limit, offset } => {
            client
                .list_posts(
                    limit.and_then(|l| l.try_into().ok()),
                    offset.and_then(|o| o.try_into().ok()),
                )
                .await?;
        }
    }

    Ok(())
}
