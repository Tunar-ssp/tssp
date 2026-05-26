//! Command-line interface definitions for `tssp`.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};

/// Top-level CLI parser.
#[derive(Debug, Clone, Parser)]
#[command(name = "tssp")]
#[command(version, about = "Self-hosted local-network file transfer client")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Output-related global flags.
    #[command(flatten)]
    pub output: OutputArgs,

    /// Logging-related global flags.
    #[command(flatten)]
    pub logging: LoggingArgs,

    /// Connection-related global flags.
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// Upload flags for the default action.
    #[command(flatten)]
    pub upload: UploadArgs,

    /// Explicit command.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Output-related global flags.
#[derive(Debug, Clone, Args)]
pub struct OutputArgs {
    /// Emit stable JSON output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-error output.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Disable terminal color.
    #[arg(long, global = true)]
    pub no_color: bool,
}

/// Logging-related global flags.
#[derive(Debug, Clone, Args)]
pub struct LoggingArgs {
    /// Emit debug logs to stderr.
    #[arg(long, global = true)]
    pub verbose: bool,
}

/// Connection-related global flags.
#[derive(Debug, Clone, Args)]
pub struct ConnectionArgs {
    /// Override daemon host.
    #[arg(long, global = true, env = "TSSP_HOST")]
    pub host: Option<String>,

    /// Override daemon port.
    #[arg(long, global = true, env = "TSSP_PORT")]
    pub port: Option<u16>,
}

/// Upload flags for the default action.
#[derive(Debug, Clone, Args)]
pub struct UploadArgs {
    /// Attach a tag during upload.
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,

    /// Pin uploaded files immediately.
    #[arg(long)]
    pub pin: bool,

    /// Store a single upload under a different logical filename.
    #[arg(long, value_name = "NEW_NAME")]
    pub rename: Option<String>,

    /// Parallelism for recursive or all-file uploads.
    #[arg(long, value_name = "N")]
    pub parallel: Option<u16>,

    /// Recursively upload a folder.
    #[arg(short = 'r', long = "recursive", value_name = "FOLDER")]
    pub recursive: Option<PathBuf>,

    /// Upload all non-hidden files in the current directory.
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// File paths for the default upload action.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// Locked CLI command surface.
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Upload a file and create a phone transfer session.
    Send(SendArgs),
    /// Create a receive session and wait for upload completion.
    Receive(ReceiveArgs),
    /// Upload the current clipboard contents.
    Paste(PasteArgs),
    /// Copy a file download URL to the clipboard.
    Copy(CopyArgs),
    /// Upload (if needed) and share a public link to a file.
    Share(ShareArgs),
    /// Download a file from the daemon.
    Pull(PullArgs),
    /// List files.
    List(ListArgs),
    /// Show the most recent uploads.
    Last(LastArgs),
    /// Show files uploaded today.
    Today,
    /// Search filenames and tags.
    Search(SearchArgs),
    /// Add tags to a file.
    Tag(TagArgs),
    /// Remove tags from a file.
    Untag(TagArgs),
    /// Pin a file to favorites.
    Pin(PinArgs),
    /// Unpin a file from favorites.
    Unpin(IdArgs),
    /// Manage pinned files.
    Pins(PinsCommand),
    /// Delete a file from the daemon.
    Remove(RemoveArgs),
    /// Show full metadata for a file.
    Info(IdArgs),
    /// Show daemon health and storage information.
    Status,
    /// Run the first-use setup wizard.
    Init,
    /// Authenticate against a remote daemon and store an API token.
    Login,
    /// Clear the saved API token.
    Logout,
    /// Show the current authenticated user.
    Whoami,
    /// Manage CLI configuration.
    Config(ConfigCommand),
    /// Manage Markdown notes.
    Note(NoteCommand),
    /// Quick-save text or clipboard to the cloud as a note.
    Cp(CpArgs),
    /// Admin: storage, folders, system, and maintenance.
    Admin(AdminCommand),
    /// Workspace status and features.
    Workspace(WorkspaceCommand),
    /// Generate a shell completion script.
    #[command(hide = true)]
    Completions(CompletionArgs),
}

/// `tssp note` subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum NoteSubcommand {
    /// Create a new note.
    Create(NoteCreateArgs),
    /// List notes.
    List(NoteListArgs),
    /// Show a note.
    Show(NoteShowArgs),
    /// Edit a note.
    Edit(NoteEditArgs),
    /// Delete a note.
    Delete(NoteDeleteArgs),
    /// Export all notes as NDJSON.
    Export(NoteExportArgs),
}

/// Wrapper for `tssp note`.
#[derive(Debug, Clone, Args)]
pub struct NoteCommand {
    /// Note action to run.
    #[command(subcommand)]
    pub action: NoteSubcommand,
}

/// Arguments for `tssp note create`.
#[derive(Debug, Clone, Args)]
pub struct NoteCreateArgs {
    /// Note body as Markdown.
    #[arg(long)]
    pub body: Option<String>,
    /// Explicit title.
    #[arg(long)]
    pub title: Option<String>,
    /// Attach a tag.
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,
    /// Pin immediately.
    #[arg(long)]
    pub pin: bool,
}

/// Arguments for `tssp note list`.
#[derive(Debug, Clone, Args)]
pub struct NoteListArgs {
    /// Filter by tag.
    #[arg(long = "tag", value_name = "NAME")]
    pub tags: Vec<String>,
    /// Page size.
    #[arg(long, value_name = "N")]
    pub limit: Option<u16>,
    /// Sort field (`updated`, `created`, `title`, with optional leading `-`).
    #[arg(long, value_name = "FIELD")]
    pub sort: Option<String>,
    /// Pinned notes only.
    #[arg(long)]
    pub pinned: bool,
}

/// Arguments for `tssp note show`.
#[derive(Debug, Clone, Args)]
pub struct NoteShowArgs {
    /// Note id.
    pub id: String,
}

/// Arguments for `tssp note edit`.
#[derive(Debug, Clone, Args)]
pub struct NoteEditArgs {
    /// Note id.
    pub id: String,
    /// Replacement body.
    #[arg(long)]
    pub body: Option<String>,
    /// New title.
    #[arg(long)]
    pub title: Option<String>,
    /// Replace all tags (repeatable; omit to leave tags unchanged).
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,
    /// Pin the note.
    #[arg(long, conflicts_with = "unpin")]
    pub pin: bool,
    /// Unpin the note.
    #[arg(long, conflicts_with = "pin")]
    pub unpin: bool,
}

/// Arguments for `tssp note export`.
#[derive(Debug, Clone, Args)]
pub struct NoteExportArgs {
    /// Write to file instead of stdout.
    #[arg(long, short = 'o', value_name = "FILE")]
    pub output: Option<std::path::PathBuf>,
}

/// Arguments for `tssp note delete`.
#[derive(Debug, Clone, Args)]
pub struct NoteDeleteArgs {
    /// Note id.
    pub id: String,
    /// Skip confirmation.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments for `tssp cp`.
#[derive(Debug, Clone, Args)]
pub struct CpArgs {
    /// Explicit title.
    #[arg(long)]
    pub title: Option<String>,
    /// Attach a tag.
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,
}

/// Arguments for `tssp send`.
#[derive(Debug, Clone, Args)]
pub struct SendArgs {
    /// File to upload and send.
    pub file: PathBuf,
    /// Force QR mode instead of active phone-session delivery.
    #[arg(long = "qr", visible_alias = "qr")]
    pub qr: bool,
    /// Attach a tag.
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,
}

/// Arguments for `tssp receive`.
#[derive(Debug, Clone, Args)]
pub struct ReceiveArgs {
    /// Also download the received file to this local path.
    #[arg(long, value_name = "PATH")]
    pub save: Option<PathBuf>,
    /// Maximum wait duration.
    #[arg(long, value_name = "DURATION")]
    pub timeout: Option<String>,
}

/// Arguments for `tssp paste`.
#[derive(Debug, Clone, Args)]
pub struct PasteArgs {
    /// Attach a tag.
    #[arg(long = "tag", short = 't', value_name = "NAME")]
    pub tags: Vec<String>,
    /// Override the generated filename.
    #[arg(long = "as", value_name = "FILENAME")]
    pub filename: Option<String>,
}

/// Arguments for `tssp copy`.
#[derive(Debug, Clone, Args)]
pub struct CopyArgs {
    /// File id.
    pub id: String,
    /// Copy a short-lived share URL instead of direct content URL.
    #[arg(long)]
    pub share: bool,
}

/// Arguments for `tssp share`.
#[derive(Debug, Clone, Args)]
pub struct ShareArgs {
    /// Local file path or existing file id.
    pub target: String,
    /// Print a terminal QR code for the link.
    #[arg(long)]
    pub qr: bool,
    /// Make the file public (default: true).
    #[arg(long, default_value_t = true)]
    pub public: bool,
    /// Attach tags when uploading a local file.
    #[arg(long = "tag", value_name = "NAME")]
    pub tags: Vec<String>,
    /// Future `WhatsApp` delivery (not implemented).
    #[arg(long, hide = true)]
    pub wp: bool,
}

/// Arguments for `tssp pull`.
#[derive(Debug, Clone, Args)]
pub struct PullArgs {
    /// File id or filename.
    pub id_or_name: String,
    /// Destination path.
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Replace an existing local file.
    #[arg(long)]
    pub overwrite: bool,
    /// Download all filename matches.
    #[arg(long)]
    pub all: bool,
}

/// Arguments for `tssp list`.
#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Filter by tag. Repeated tags use AND semantics.
    #[arg(long = "tag", value_name = "NAME")]
    pub tags: Vec<String>,
    /// Filter by MIME prefix.
    #[arg(long = "type", value_name = "MIME_PREFIX")]
    pub mime_prefix: Option<String>,
    /// Filter by relative duration or ISO timestamp.
    #[arg(long, value_name = "WHEN")]
    pub since: Option<String>,
    /// Page size.
    #[arg(long, value_name = "N")]
    pub limit: Option<u16>,
    /// Sort field.
    #[arg(long, value_name = "FIELD")]
    pub sort: Option<String>,
    /// Show pinned files only.
    #[arg(long)]
    pub pinned: bool,
    /// Cursor for the next page.
    #[arg(long, value_name = "CURSOR")]
    pub page: Option<String>,
}

/// Arguments for `tssp last`.
#[derive(Debug, Clone, Args)]
pub struct LastArgs {
    /// Number of uploads to show.
    #[arg(default_value_t = 10)]
    pub count: u16,
}

/// Arguments for `tssp search`.
#[derive(Debug, Clone, Args)]
pub struct SearchArgs {
    /// Query text.
    pub query: String,
    /// Maximum number of results.
    #[arg(long, value_name = "N")]
    pub limit: Option<u16>,
    /// Restrict to files with this tag.
    #[arg(long = "tag", value_name = "NAME")]
    pub tag: Option<String>,
}

/// Arguments for tag mutations.
#[derive(Debug, Clone, Args)]
pub struct TagArgs {
    /// File id.
    pub id: String,
    /// One or more tag names.
    pub tags: Vec<String>,
}

/// Arguments for `tssp pin`.
#[derive(Debug, Clone, Args)]
pub struct PinArgs {
    /// File id.
    pub id: String,
    /// Insert at a specific pin position.
    #[arg(long, value_name = "N")]
    pub position: Option<u32>,
}

/// `tssp pins` subcommands.
#[derive(Debug, Clone, Args)]
pub struct PinsCommand {
    /// Action to perform on pins.
    #[command(subcommand)]
    pub action: PinsAction,
}

/// Pin list and reorder actions.
#[derive(Debug, Clone, Subcommand)]
pub enum PinsAction {
    /// List pinned files.
    List,
    /// Reorder pinned files (pass IDs in the new desired order).
    Reorder(ReorderArgs),
}

/// Arguments for `tssp pins reorder`.
#[derive(Debug, Clone, Args)]
pub struct ReorderArgs {
    /// File IDs in the exact order they should appear.
    #[arg(required = true)]
    pub ids: Vec<String>,
}

/// Arguments for `tssp remove`.
#[derive(Debug, Clone, Args)]
pub struct RemoveArgs {
    /// File id.
    pub id: String,
    /// Skip confirmation.
    #[arg(long)]
    pub yes: bool,
}

/// Single id argument.
#[derive(Debug, Clone, Args)]
pub struct IdArgs {
    /// File id.
    pub id: String,
}

/// Wrapper for `tssp admin`.
#[derive(Debug, Clone, Args)]
pub struct AdminCommand {
    /// Admin action.
    #[command(subcommand)]
    pub action: AdminAction,
}

/// `tssp admin` subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum AdminAction {
    /// Dashboard summary (files, notes, storage).
    Overview,
    /// Host CPU, memory, and disk metrics.
    System,
    /// List objects (optionally by folder or MIME prefix).
    Files(AdminFilesArgs),
    /// List virtual folders and file counts.
    Folders,
    /// Delete a file by id (admin endpoint).
    Delete(AdminDeleteArgs),
    /// Show corrupt blob count from last integrity scan.
    Corrupt,
    /// Remove staged temp upload files.
    CleanupTemp,
    /// Show session cleanup status.
    CleanupSessions,
    /// User account management.
    Users(AdminUsersCommand),
    /// Trusted device management.
    Devices(AdminDevicesCommand),
}

/// Workspace management and status.
#[derive(Debug, Clone, Args)]
pub struct WorkspaceCommand {
    /// Workspace ID.
    #[arg(value_name = "ID")]
    pub workspace_id: String,
    /// Workspace action.
    #[command(subcommand)]
    pub action: WorkspaceAction,
}

/// `tssp workspace` subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum WorkspaceAction {
    /// Show terminal session status and availability.
    TerminalStatus,
    /// Show language server availability.
    LspStatus,
    /// Show git repository status.
    GitStatus,
}

/// `tssp admin users` subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum AdminUsersAction {
    /// List all users.
    List,
    /// Create a user.
    Create(AdminUserCreateArgs),
    /// Delete a user by id.
    Delete(AdminUserIdArgs),
    /// Set user role (`admin` or `user`).
    SetRole(AdminUserRoleArgs),
    /// Reset a user's access code.
    ResetCode(AdminUserResetCodeArgs),
}

/// Wrapper for `tssp admin users`.
#[derive(Debug, Clone, Args)]
pub struct AdminUsersCommand {
    /// User management action.
    #[command(subcommand)]
    pub action: AdminUsersAction,
}

/// Wrapper for `tssp admin devices`.
#[derive(Debug, Clone, Args)]
pub struct AdminDevicesCommand {
    /// Device management action.
    #[command(subcommand)]
    pub action: AdminDevicesAction,
}

/// `tssp admin devices` subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum AdminDevicesAction {
    /// List trusted devices.
    List,
    /// Revoke a device by token.
    Revoke(AdminDeviceRevokeArgs),
    /// Revoke all devices for a user.
    RevokeUser(AdminUserIdArgs),
}

/// Arguments for `tssp admin users create`.
#[derive(Debug, Clone, Args)]
pub struct AdminUserCreateArgs {
    /// Display name.
    pub name: String,
    /// Access code.
    pub code: String,
    /// Role (`admin` or `user`, default `user`).
    #[arg(long, default_value = "user")]
    pub role: String,
}

/// Arguments for `tssp admin users delete` / devices revoke-user.
#[derive(Debug, Clone, Args)]
pub struct AdminUserIdArgs {
    /// User id.
    pub id: String,
}

/// Arguments for `tssp admin users set-role`.
#[derive(Debug, Clone, Args)]
pub struct AdminUserRoleArgs {
    /// User id.
    pub id: String,
    /// New role (`admin` or `user`).
    pub role: String,
}

/// Arguments for `tssp admin users reset-code`.
#[derive(Debug, Clone, Args)]
pub struct AdminUserResetCodeArgs {
    /// User id.
    pub id: String,
    /// New access code.
    pub code: String,
}

/// Arguments for `tssp admin devices revoke`.
#[derive(Debug, Clone, Args)]
pub struct AdminDeviceRevokeArgs {
    /// Device token.
    pub token: String,
}

/// Arguments for `tssp admin files`.
#[derive(Debug, Clone, Args)]
pub struct AdminFilesArgs {
    /// Max results (default 100).
    #[arg(long, default_value_t = 100)]
    pub limit: u64,
    /// Folder prefix filter.
    #[arg(long)]
    pub folder: Option<String>,
    /// MIME prefix (e.g. `image` → `image/`).
    #[arg(long, name = "type")]
    pub mime_type: Option<String>,
}

/// Arguments for `tssp admin delete`.
#[derive(Debug, Clone, Args)]
pub struct AdminDeleteArgs {
    /// File id to delete.
    pub id: String,
}

/// `tssp config` subcommands.
#[derive(Debug, Clone, Args)]
pub struct ConfigCommand {
    /// Configuration action.
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Configuration mutation and lookup actions.
#[derive(Debug, Clone, Subcommand)]
pub enum ConfigAction {
    /// Set a config key.
    Set(ConfigSetArgs),
    /// Get one config key or all config.
    Get(ConfigGetArgs),
    /// Remove a config key.
    Unset(ConfigUnsetArgs),
    /// Print the config file path.
    Path,
}

/// Arguments for `tssp config set`.
#[derive(Debug, Clone, Args)]
pub struct ConfigSetArgs {
    /// Config key.
    pub key: String,
    /// Config value.
    pub value: String,
}

/// Arguments for `tssp config get`.
#[derive(Debug, Clone, Args)]
pub struct ConfigGetArgs {
    /// Optional config key.
    pub key: Option<String>,
}

/// Arguments for `tssp config unset`.
#[derive(Debug, Clone, Args)]
pub struct ConfigUnsetArgs {
    /// Config key.
    pub key: String,
}

/// Arguments for hidden completion generation.
#[derive(Debug, Clone, Args)]
pub struct CompletionArgs {
    /// Target shell.
    pub shell: CompletionShell,
}

/// Supported completion shells.
#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CompletionShell {
    /// Bash.
    Bash,
    /// Z shell.
    Zsh,
    /// Fish shell.
    Fish,
    /// `PowerShell`.
    Powershell,
}

impl CompletionShell {
    fn shell(self) -> Shell {
        match self {
            Self::Bash => Shell::Bash,
            Self::Zsh => Shell::Zsh,
            Self::Fish => Shell::Fish,
            Self::Powershell => Shell::PowerShell,
        }
    }
}

/// Generates a completion script for the requested shell.
#[must_use]
pub fn generate_completion(shell: CompletionShell) -> Vec<u8> {
    let mut command = Cli::command();
    let mut output = Vec::new();
    generate(shell.shell(), &mut command, "tssp", &mut output);
    output
}

/// Parses command-line arguments from any iterator.
///
/// # Errors
///
/// Returns [`clap::Error`] when the provided arguments are not valid for the
/// command definition.
pub fn parse_from<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Cli::try_parse_from(args)
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::{generate_completion, parse_from, Cli, Command, CompletionShell};

    #[test]
    fn clap_command_tree_is_internally_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_status_command() {
        let parsed = parse_from(["tssp", "status"]);

        assert!(matches!(
            parsed,
            Ok(Cli {
                command: Some(Command::Status),
                ..
            })
        ));
    }

    #[test]
    fn parses_default_upload_arguments() {
        let parsed = parse_from(["tssp", "--tag", "docs", "--pin", "README.md"]);

        assert!(matches!(
            parsed,
            Ok(Cli {
                upload: super::UploadArgs { pin: true, .. },
                command: None,
                ..
            })
        ));
    }

    #[test]
    fn completion_generation_contains_known_command() {
        let script = generate_completion(CompletionShell::Bash);
        let text = String::from_utf8(script);

        assert!(matches!(text, Ok(value) if value.contains("status")));
    }
}
