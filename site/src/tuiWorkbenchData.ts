import type {
  ComponentBlueprint,
  FolderBrowserRecipe,
  InteractionContract,
  LayoutRecipe,
  ResourceKit,
  SchematicRecipe,
  TableRecipe,
  TabRecipe,
  TuiPattern,
} from "./tuiWorkbenchTypes";

export const tuiPatterns: TuiPattern[] = [
  {
    pattern: "Research table",
    module: "ratatui::widgets::Table",
    useCase: "Benchmarks, traces, issue queues, model runs",
    rustHook: "Vec<RowModel> + sort key + selection state",
    dotmaxHook: "braille sparklines, color severity, density bars",
    priority: "Core",
  },
  {
    pattern: "Folder navigator",
    module: "List + Tree state",
    useCase: "Project browsers, artifact inspectors, log bundles",
    rustHook: "arena tree + expanded HashSet<PathBuf>",
    dotmaxHook: "tiny previews for images, charts, diffs, frame packs",
    priority: "Core",
  },
  {
    pattern: "Command tabs",
    module: "Tabs + block chrome",
    useCase: "Mode switching without modal sprawl",
    rustHook: "enum View { Table, Files, Graph, Logs }",
    dotmaxHook: "per-tab terminal thumbnail and progress summary",
    priority: "Core",
  },
  {
    pattern: "Schematic panel",
    module: "Canvas-style layout",
    useCase: "Pipelines, service maps, job DAGs, deploy flows",
    rustHook: "node list + edge list + focused node id",
    dotmaxHook: "braille connectors, animated edge pulses, health colors",
    priority: "Next",
  },
  {
    pattern: "Inspector drawer",
    module: "Paragraph + Table + Gauge",
    useCase: "Selected row details, metadata, logs, actions",
    rustHook: "Option<Selection> -> derived detail model",
    dotmaxHook: "compact histograms and confidence bands",
    priority: "Next",
  },
  {
    pattern: "Status command bar",
    module: "Layout footer",
    useCase: "Key hints, active filter, queue status, latency",
    rustHook: "Command registry + keyboard resolver",
    dotmaxHook: "animated loader glyphs, frame-time meter",
    priority: "Core",
  },
  {
    pattern: "Artifact gallery",
    module: "Grid of previews",
    useCase: "Images, generated assets, fixtures, model outputs",
    rustHook: "paged Vec<Artifact> + preview cache",
    dotmaxHook: "image-to-braille thumbs and GIF/APNG frame strips",
    priority: "Lab",
  },
  {
    pattern: "Run comparison matrix",
    module: "Table + heatmap cells",
    useCase: "A/B tests, CI jobs, benchmark sweeps",
    rustHook: "column groups + derived deltas",
    dotmaxHook: "RGB cell heat, bar glyphs, sparkline deltas",
    priority: "Next",
  },
];

export const tableRecipes: TableRecipe[] = [
  {
    id: "sortable-column-schema",
    title: "Column schema",
    fit: "shadcn-style data-table columns for benchmark rows, trace rows, and issue queues.",
    column: "Column<RowModel>",
    state: ["id", "label", "width", "sort_value", "render_cell"],
    filters: ["sort asc/desc", "hide column", "pin primary column"],
    preview: ["name          latency   status", "parser/http   7.4 ms    pass", "image/png    11.2 ms    warn"],
    snippet: `struct Column<T> {
    id: &'static str,
    label: &'static str,
    width: u16,
    sort_value: fn(&T) -> SortValue,
    render_cell: fn(&T) -> Cell<'static>,
}

let latency = Column {
    id: "latency",
    label: "Latency",
    width: 10,
    sort_value: |row: &RunRow| SortValue::Millis(row.latency_ms),
    render_cell: |row: &RunRow| Cell::from(format!("{:.1} ms", row.latency_ms)),
};`,
  },
  {
    id: "faceted-filter-bar",
    title: "Faceted filter bar",
    fit: "dataset filters that stay visible above a research table without becoming a modal.",
    column: "FilterState",
    state: ["text query", "status facets", "owner facets", "active tags"],
    filters: ["status", "owner", "feature", "date bucket"],
    preview: ["filters  status:warn owner:ci tag:image", "visible  42 / 586 rows", "press / search   f facets   esc clear"],
    snippet: `#[derive(Default)]
struct FilterState {
    query: String,
    statuses: BTreeSet<RunStatus>,
    owners: BTreeSet<String>,
    tags: BTreeSet<String>,
}

fn row_matches(row: &RunRow, filters: &FilterState) -> bool {
    let status_ok = filters.statuses.is_empty() || filters.statuses.contains(&row.status);
    let owner_ok = filters.owners.is_empty() || filters.owners.contains(&row.owner);
    let text_ok = filters.query.is_empty() || row.search_text().contains(&filters.query);
    status_ok && owner_ok && text_ok
}`,
  },
  {
    id: "row-action-menu",
    title: "Row action menu",
    fit: "copy id, open source, rerun, compare, and export actions scoped to the selected row.",
    column: "RowAction",
    state: ["selected row id", "available actions", "disabled reasons", "last command result"],
    filters: ["action scope", "permission", "row status"],
    preview: ["selected parser/http", "actions  open  copy  rerun  compare", "enter run   c copy   o open"],
    snippet: `enum RowAction {
    CopyId,
    OpenSource,
    Rerun,
    CompareWithBaseline,
}

fn actions_for(row: &RunRow) -> Vec<RowAction> {
    let mut actions = vec![RowAction::CopyId, RowAction::OpenSource];
    if row.can_rerun {
        actions.push(RowAction::Rerun);
    }
    actions.push(RowAction::CompareWithBaseline);
    actions
}`,
  },
  {
    id: "dotmax-sparkline-cell",
    title: "dotmax sparkline cell",
    fit: "dense visual history inside a ratatui table cell for latency, memory, fps, and queue depth.",
    column: "SparkCell",
    state: ["samples", "min/max", "threshold", "color ramp"],
    filters: ["trend up", "trend down", "outlier", "flat"],
    preview: ["latency  7.4 ms  ⢀⣀⣤⣶⣶⣤", "memory   122 mb  ⣶⣶⣤⣀⡀", "fps      59.8    ⣀⣀⣀⣀⣀"],
    snippet: `struct SparkCell {
    samples: Vec<f32>,
    warning_at: f32,
}

fn render_spark_cell(cell: &SparkCell) -> Vec<String> {
    let mut grid = BrailleGrid::new(12, 1).expect("valid sparkline grid");
    // Map samples into dot columns, then let dotmax render compact braille.
    draw_metric_sparkline(&mut grid, &cell.samples, cell.warning_at);
    grid.rows()
}`,
  },
  {
    id: "pinned-summary-row",
    title: "Pinned summary row",
    fit: "a sticky table footer for totals, selected-row aggregates, benchmark deltas, and active filter counts.",
    column: "SummaryRow",
    state: ["visible count", "selected count", "aggregate latency", "worst status"],
    filters: ["all rows", "visible rows", "selected rows"],
    preview: ["summary  42 visible  3 failed  p95 12.8 ms", "delta    +1.6 ms vs baseline", "footer derives from filtered rows"],
    snippet: `struct SummaryRow {
    visible: usize,
    failed: usize,
    p95_latency_ms: f32,
    delta_ms: f32,
}

fn summarize(rows: &[RunRow]) -> SummaryRow {
    SummaryRow {
        visible: rows.len(),
        failed: rows.iter().filter(|row| row.status.is_failed()).count(),
        p95_latency_ms: percentile(rows, 0.95),
        delta_ms: baseline_delta(rows),
    }
}`,
  },
];

export const folderRecipes = [
  {
    path: "src/app.rs",
    label: "App shell",
    summary: "Owns event loop, global commands, and view routing.",
    recipe: "Keep top-level state boring: input, selected view, layout, theme, and async inbox.",
  },
  {
    path: "src/views/table.rs",
    label: "Research table",
    summary: "Sortable rows with pinned summary and right-side inspector.",
    recipe: "Represent rows as data first, then render cells from pure formatters.",
  },
  {
    path: "src/views/files.rs",
    label: "Folder tree",
    summary: "Expanded folders, selected artifact, preview pane.",
    recipe: "Store expansion separately from filesystem data so refreshes do not collapse the UI.",
  },
  {
    path: "src/views/schematic.rs",
    label: "Schematic",
    summary: "Nodes, edges, health, and focused module detail.",
    recipe: "Use stable node ids and deterministic layout before adding animation.",
  },
  {
    path: "src/components/status_bar.rs",
    label: "Command bar",
    summary: "Keyboard hints, active filters, job state, frame budget.",
    recipe: "Render commands from a registry so help text and input handling cannot drift.",
  },
];

export const folderBrowserRecipes: FolderBrowserRecipe[] = [
  {
    id: "arena-tree-state",
    title: "Arena tree state",
    fit: "Project explorers and artifact browsers that need stable ids across refreshes.",
    state: ["nodes: Vec<TreeNode>", "expanded: HashSet<NodeId>", "selected: NodeId", "visible: Vec<NodeId>"],
    events: ["scan filesystem", "toggle expand", "move visible selection", "preserve ids on refresh"],
    preview: ["src/", "  app.rs", "  views/", "    files.rs  selected", "  components/"],
    snippet: `type NodeId = usize;

struct TreeNode {
    id: NodeId,
    parent: Option<NodeId>,
    path: PathBuf,
    depth: usize,
    is_dir: bool,
}

struct TreeState {
    nodes: Vec<TreeNode>,
    expanded: HashSet<NodeId>,
    selected: NodeId,
    visible: Vec<NodeId>,
}`,
  },
  {
    id: "preview-cache",
    title: "Preview cache",
    fit: "Fast previews for images, frame packs, logs, screenshots, and generated artifacts.",
    state: ["cache: LruCache<PathBuf, PreviewModel>", "stale paths", "loader jobs", "last rendered frame"],
    events: ["select file", "load preview async", "invalidate on mtime change", "fallback to text summary"],
    preview: ["preview src/gallery/frame.json", "kind frame-pack   frames 48", "render cached     2.1 ms", "dotmax thumb      ready"],
    snippet: `enum PreviewModel {
    Text(Vec<String>),
    ImageRows(Vec<String>),
    FramePack { frames: usize, rows: Vec<String> },
    Unsupported(String),
}

fn preview_for(path: &Path, cache: &mut PreviewCache) -> PreviewModel {
    cache.get(path).cloned().unwrap_or_else(|| PreviewModel::Text(vec![
        format!("{} not cached yet", path.display()),
    ]))
}`,
  },
  {
    id: "filesystem-refresh",
    title: "Filesystem refresh",
    fit: "Refresh project trees without collapsing open folders or losing selected artifacts.",
    state: ["previous path ids", "expanded paths", "selected path", "new scan tree"],
    events: ["r refresh", "debounced watcher", "restore expansion", "clamp selection"],
    preview: ["refresh complete", "expanded kept  7", "selected kept  src/views/files.rs", "new files      3"],
    snippet: `fn refresh_tree(state: &mut FileBrowser, scanned: Vec<TreeNode>) {
    let selected_path = state.selected_path();
    let expanded_paths = state.expanded_paths();

    state.nodes = scanned;
    state.expanded = state.ids_for_paths(&expanded_paths);
    state.selected = state
        .id_for_path(&selected_path)
        .unwrap_or_else(|| state.first_visible_id());
    state.rebuild_visible();
}`,
  },
  {
    id: "split-preview-layout",
    title: "Split preview layout",
    fit: "Three-pane TUI file explorers: tree, preview, metadata/actions.",
    state: ["tree width", "preview min", "metadata width", "footer commands"],
    events: ["resize", "toggle metadata", "focus preview", "copy path"],
    preview: ["| tree 28 | preview min 48 | meta 26 |", "| files   | braille image   | tags    |", "| footer: enter open  p preview  y copy |"],
    snippet: `let shell = Layout::vertical([
    Constraint::Min(10),
    Constraint::Length(3),
]).split(area);

let panes = Layout::horizontal([
    Constraint::Length(28),
    Constraint::Min(48),
    Constraint::Length(26),
]).split(shell[0]);`,
  },
];

export const schematicModules = [
  ["Input", "keys, mouse, resize"],
  ["State", "selection, filters, cache"],
  ["Layout", "split, tabs, panels"],
  ["Render", "ratatui widgets"],
  ["dotmax", "braille previews"],
  ["Actions", "commands, jobs, io"],
];

export const schematicRecipes: SchematicRecipe[] = [
  {
    id: "stable-node-model",
    title: "Stable node model",
    fit: "Service maps, build graphs, deploy flows, and job DAGs that must survive refreshes.",
    nodes: ["NodeId", "label", "kind", "status", "position"],
    signals: ["focus node", "pin path", "refresh graph", "open inspector"],
    preview: ["ingest -> parse -> render", "   |       |       |", " cache <- diff <- export"],
    snippet: `#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct NodeId(u64);

struct SchematicNode {
    id: NodeId,
    label: String,
    kind: NodeKind,
    status: Health,
    position: Option<(u16, u16)>,
}

struct SchematicState {
    nodes: Vec<SchematicNode>,
    edges: Vec<SchematicEdge>,
    focused: NodeId,
}`,
  },
  {
    id: "deterministic-layout-pass",
    title: "Deterministic layout pass",
    fit: "Graph UIs that need predictable screenshots, tests, and keyboard navigation.",
    nodes: ["topological layers", "stable sort key", "row/column slots", "collision pass"],
    signals: ["resize", "node added", "layer changed", "layout dirty"],
    preview: ["layer 0    input", "layer 1    normalize  cache", "layer 2    render     export"],
    snippet: `fn layout_layers(graph: &mut SchematicState) {
    let mut layers = topological_layers(&graph.nodes, &graph.edges);
    for layer in &mut layers {
        layer.sort_by_key(|node_id| graph.label_for(*node_id).to_owned());
    }

    for (column, layer) in layers.iter().enumerate() {
        for (row, node_id) in layer.iter().enumerate() {
            graph.set_position(*node_id, (column as u16 * 18, row as u16 * 5));
        }
    }
}`,
  },
  {
    id: "health-propagation",
    title: "Health propagation",
    fit: "Pipelines where failures, stale caches, and warnings should color downstream context.",
    nodes: ["local health", "incoming edge health", "derived health", "last change"],
    signals: ["job failed", "cache stale", "warning threshold", "health acknowledged"],
    preview: ["source ok  -> parser warn -> render warn", "cache stale -> diff warn", "export blocked by failed input"],
    snippet: `fn derived_health(node: NodeId, graph: &SchematicState) -> Health {
    graph
        .incoming(node)
        .map(|edge| edge.health)
        .chain(std::iter::once(graph.node(node).status))
        .max()
        .unwrap_or(Health::Unknown)
}`,
  },
  {
    id: "edge-routing",
    title: "Braille edge routing",
    fit: "Dense terminal schematics that need readable connectors inside narrow panes.",
    nodes: ["source port", "target port", "route points", "pulse phase"],
    signals: ["focused path", "animated edge", "hidden edge", "edge label"],
    preview: ["A ──╮", "   ╰── B ── C", "      ╰── D"],
    snippet: `struct EdgeRoute {
    from: NodeId,
    to: NodeId,
    points: Vec<(u16, u16)>,
    focused: bool,
}

fn draw_route(grid: &mut BrailleGrid, route: &EdgeRoute, color: Color) {
    for pair in route.points.windows(2) {
        let [from, to] = [pair[0], pair[1]];
        draw_line_colored(grid, from.0.into(), from.1.into(), to.0.into(), to.1.into(), color, None);
    }
}`,
  },
  {
    id: "node-action-panel",
    title: "Node action panel",
    fit: "Focused graph nodes with scoped actions: rerun, open logs, copy id, jump source.",
    nodes: ["focused node", "action list", "disabled reason", "last result"],
    signals: ["enter inspect", "a action", "l logs", "r rerun"],
    preview: ["focused render-stage", "actions  open logs  rerun  copy id", "status   warn  p95 12.8 ms"],
    snippet: `enum NodeAction {
    OpenLogs,
    Rerun,
    CopyNodeId,
    JumpToSource,
}

fn node_actions(node: &SchematicNode) -> Vec<NodeAction> {
    let mut actions = vec![NodeAction::OpenLogs, NodeAction::CopyNodeId];
    if node.status.can_rerun() {
        actions.push(NodeAction::Rerun);
    }
    actions.push(NodeAction::JumpToSource);
    actions
}`,
  },
];

export const tabRecipes: TabRecipe[] = [
  {
    id: "typed-view-router",
    title: "Typed view router",
    fit: "Main app shells that need table, folders, schematic, logs, and jobs without stringly state.",
    views: ["Research", "Files", "Graph", "Logs", "Jobs"],
    state: ["active: View", "last_active: View", "per-view state structs", "command scope from active view"],
    keymap: "1..5 jump, tab next, shift+tab previous",
    snippet: `#[derive(Clone, Copy, Eq, PartialEq)]
enum View {
    Research,
    Files,
    Graph,
    Logs,
    Jobs,
}

impl View {
    const ALL: [View; 5] = [Self::Research, Self::Files, Self::Graph, Self::Logs, Self::Jobs];

    fn next(self) -> Self {
        let index = Self::ALL.iter().position(|view| *view == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}`,
  },
  {
    id: "tab-scoped-command-map",
    title: "Tab-scoped command map",
    fit: "Command palettes and footer hints that stay honest as users switch modes.",
    views: ["Table commands", "Folder commands", "Graph commands", "Global commands"],
    state: ["registry: Vec<Command>", "scope: CommandScope", "footer hints from active scope", "palette filter"],
    keymap: "?, ctrl+k, enter to dispatch",
    snippet: `struct Command {
    id: &'static str,
    scope: CommandScope,
    keys: &'static [&'static str],
}

fn visible_commands(app: &App) -> impl Iterator<Item = &Command> {
    app.commands.iter().filter(|command| command.scope.matches(app.view))
}`,
  },
  {
    id: "stateful-tabs",
    title: "Stateful child views",
    fit: "Developer tools where leaving a tab must preserve selection, filters, scroll, and async job handles.",
    views: ["table selected row", "tree expansion", "graph focus", "log scroll"],
    state: ["table: TableState", "files: FolderTree", "graph: GraphState", "logs: LogState"],
    keymap: "switching tabs never resets child state",
    snippet: `struct App {
    view: View,
    table: TableState<RowModel>,
    files: FolderTree,
    graph: GraphState,
    logs: LogState,
}

fn set_view(app: &mut App, next: View) {
    app.view = next;
}`,
  },
  {
    id: "preview-backed-tabs",
    title: "Preview-backed tabs",
    fit: "Tabs that show dense dotmax status: sparkline, loader, health glyph, or frame-pack thumbnail.",
    views: ["Benchmarks", "Artifacts", "Pipeline", "Queue"],
    state: ["tab summaries", "frame cache ids", "dirty flags", "last render timestamp"],
    keymap: "tab strip is navigation plus live status",
    snippet: `struct TabSummary {
    view: View,
    label: &'static str,
    badge: Option<String>,
    preview: Option<DotmaxFrameId>,
}

fn tab_label(summary: &TabSummary) -> String {
    match &summary.badge {
        Some(badge) => format!("{} {}", summary.label, badge),
        None => summary.label.into(),
    }
}`,
  },
];

export const componentBlueprints: ComponentBlueprint[] = [
  {
    id: "data-table",
    title: "Data table with selection",
    problem: "Sortable research rows with pinned state and a detail drawer.",
    state: ["rows: Vec<RowModel>", "selected: usize", "sort: SortKey", "filter: String"],
    keys: ["j/k or down/up: move", "s: cycle sort", "/: focus filter", "enter: inspect row"],
    code: `struct TableState<T> {
    rows: Vec<T>,
    selected: usize,
    sort: SortKey,
    filter: String,
}

impl<T> TableState<T> {
    fn selected(&self) -> Option<&T> {
        self.rows.get(self.selected)
    }
}`,
  },
  {
    id: "folder-tree",
    title: "Folder tree with preview",
    problem: "Project and artifact browsing without losing expanded folders on refresh.",
    state: ["nodes: Vec<TreeNode>", "expanded: HashSet<NodeId>", "selected: NodeId", "preview_cache: LruCache<NodeId, FramePack>"],
    keys: ["h/l: collapse or expand", "j/k: move", "p: toggle preview", "r: refresh tree"],
    code: `struct FolderTree {
    nodes: Vec<TreeNode>,
    expanded: HashSet<NodeId>,
    selected: NodeId,
}

fn toggle(tree: &mut FolderTree, id: NodeId) {
    if !tree.expanded.insert(id) {
        tree.expanded.remove(&id);
    }
}`,
  },
  {
    id: "tab-shell",
    title: "Tabs as typed views",
    problem: "Mode switching that keeps state explicit and testable.",
    state: ["view: View", "table: TableState<Row>", "files: FolderTree", "schematic: GraphState"],
    keys: ["1..4: jump views", "tab: next view", "shift+tab: previous view", "?: show command map"],
    code: `enum View {
    Table,
    Files,
    Schematic,
    Logs,
}

struct App {
    view: View,
    commands: CommandRegistry,
}`,
  },
  {
    id: "inspector",
    title: "Inspector drawer",
    problem: "Selected-row metadata, logs, metrics, and actions in one predictable panel.",
    state: ["selection: Option<EntityId>", "sections: Vec<InspectorSection>", "active_section: usize"],
    keys: ["[: previous section", "]: next section", "c: copy id", "o: open source"],
    code: `fn inspector_model(app: &App) -> Option<InspectorModel> {
    let selected = app.selection?;
    Some(InspectorModel {
        title: selected.label(),
        metrics: selected.metrics(),
        actions: selected.actions(),
    })
}`,
  },
  {
    id: "schematic",
    title: "Schematic graph panel",
    problem: "Pipelines, services, and job DAGs with stable node focus.",
    state: ["nodes: Vec<Node>", "edges: Vec<Edge>", "focused: NodeId", "health: HashMap<NodeId, Health>"],
    keys: ["arrow keys: move focus", "enter: inspect node", "space: pin path", "a: run action"],
    code: `struct GraphState {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    focused: NodeId,
}

fn visible_edges(graph: &GraphState) -> impl Iterator<Item = &Edge> {
    graph.edges.iter().filter(|edge| edge.visible)
}`,
  },
];

export const resourceKits: ResourceKit[] = [
  {
    id: "research-dashboard",
    title: "Research dashboard",
    fit: "Benchmark runs, model evals, trace tables, and experiment comparison.",
    dependencies: ["ratatui", "crossterm", "dotmax", "color-eyre", "serde"],
    files: ["src/app.rs", "src/views/runs_table.rs", "src/components/inspector.rs", "src/components/sparkline.rs"],
    checklist: ["sortable rows", "filter input", "selected-row inspector", "dotmax sparkline preview"],
    command: "cargo add ratatui crossterm color-eyre serde dotmax",
    code: `enum ResearchMsg {
    Sort(SortKey),
    Filter(String),
    Select(usize),
    OpenArtifact(PathBuf),
}`,
  },
  {
    id: "artifact-explorer",
    title: "Artifact explorer",
    fit: "Project folders, generated images, frame packs, logs, fixtures, and screenshots.",
    dependencies: ["ratatui", "ignore", "walkdir", "dotmax --features image", "lru"],
    files: ["src/views/files.rs", "src/components/tree.rs", "src/components/preview.rs", "src/cache.rs"],
    checklist: ["expanded folder state", "preview cache", "image-to-braille thumb", "metadata drawer"],
    command: "cargo add ratatui ignore walkdir lru dotmax --features image",
    code: `struct ArtifactPreview {
    path: PathBuf,
    kind: ArtifactKind,
    rows: Vec<String>,
    stale: bool,
}`,
  },
  {
    id: "ops-schematic",
    title: "Ops schematic",
    fit: "Service maps, job DAGs, dataflow diagrams, and deploy pipelines.",
    dependencies: ["ratatui", "petgraph", "dotmax", "tokio", "tracing"],
    files: ["src/views/schematic.rs", "src/graph/layout.rs", "src/graph/health.rs", "src/actions.rs"],
    checklist: ["stable node ids", "edge focus", "health color map", "action registry"],
    command: "cargo add ratatui petgraph tokio tracing dotmax",
    code: `struct SchematicNode {
    id: NodeId,
    label: String,
    health: Health,
    preview: Option<FramePackId>,
}`,
  },
  {
    id: "command-lab",
    title: "Command lab",
    fit: "Developer tools with tabs, keymaps, command palette, and async jobs.",
    dependencies: ["ratatui", "crossterm", "tokio", "dotmax", "strum"],
    files: ["src/commands.rs", "src/keymap.rs", "src/views/mod.rs", "src/jobs.rs"],
    checklist: ["typed view enum", "command registry", "job status table", "footer key hints"],
    command: "cargo add ratatui crossterm tokio strum dotmax",
    code: `struct Command {
    id: &'static str,
    keys: &'static [&'static str],
    run: fn(&mut App) -> CommandResult,
}`,
  },
];

export const interactionContracts: InteractionContract[] = [
  {
    id: "table.move-selection",
    scope: "Research table",
    keys: "j/k, up/down",
    stateChange: "selected row changes, inspector derives from selected row",
    test: "pressing down at row 0 selects row 1 and preserves sort/filter",
    snippet: `app.dispatch(Command::MoveSelection(1));
assert_eq!(app.table.selected, 1);
assert_eq!(app.view, View::ResearchTable);`,
  },
  {
    id: "table.apply-filter",
    scope: "Research table",
    keys: "/, esc, enter",
    stateChange: "filter text updates visible rows, selected index clamps",
    test: "filtering to fewer rows never leaves selected out of bounds",
    snippet: `app.dispatch(Command::SetFilter("failed".into()));
assert!(app.table.selected < app.table.visible_rows().len());`,
  },
  {
    id: "files.toggle-folder",
    scope: "Folder tree",
    keys: "h/l, enter",
    stateChange: "expanded set toggles without mutating source tree",
    test: "refresh keeps expansion state for stable node ids",
    snippet: `let before = app.files.expanded.clone();
app.dispatch(Command::RefreshFiles);
assert_eq!(app.files.expanded, before);`,
  },
  {
    id: "tabs.switch-view",
    scope: "Tab shell",
    keys: "tab, shift+tab, 1..5",
    stateChange: "active view changes, child view state is retained",
    test: "switching away from table and back keeps row selection",
    snippet: `let selected = app.table.selected;
app.dispatch(Command::SetView(View::Files));
app.dispatch(Command::SetView(View::ResearchTable));
assert_eq!(app.table.selected, selected);`,
  },
  {
    id: "schematic.focus-node",
    scope: "Schematic",
    keys: "arrows, enter",
    stateChange: "focused node id updates, inspector receives node model",
    test: "focus only lands on visible nodes and opens node inspector",
    snippet: `app.dispatch(Command::FocusNode(next));
assert!(app.graph.visible_nodes().any(|node| node.id == app.graph.focused));`,
  },
  {
    id: "command.open-palette",
    scope: "Command bar",
    keys: "?, ctrl+k",
    stateChange: "palette opens with command registry filtered by current view",
    test: "palette only shows commands valid for active view",
    snippet: `app.dispatch(Command::OpenPalette);
assert!(app.palette.commands.iter().all(|cmd| cmd.scope.matches(app.view)));`,
  },
];

export const layoutRecipes: LayoutRecipe[] = [
  {
    id: "table-inspector-shell",
    title: "Table + inspector shell",
    fit: "Research rows, benchmark sweeps, CI runs, trace lists",
    regions: ["header 3", "table min 40", "inspector 36", "footer 3"],
    snippet: `let vertical = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(12),
    Constraint::Length(3),
]).split(area);

let body = Layout::horizontal([
    Constraint::Min(40),
    Constraint::Length(36),
]).split(vertical[1]);`,
  },
  {
    id: "folder-preview-shell",
    title: "Folder + preview shell",
    fit: "Artifact browsers, source explorers, image/frame-pack viewers",
    regions: ["tree 32", "preview min 48", "metadata 28", "footer 3"],
    snippet: `let body = Layout::horizontal([
    Constraint::Length(32),
    Constraint::Min(48),
    Constraint::Length(28),
]).split(content_area);`,
  },
  {
    id: "tabbed-command-lab",
    title: "Tabbed command lab",
    fit: "Developer tools with modes, jobs, command palette, and logs",
    regions: ["tabs 3", "active view min 20", "job strip 7", "command bar 3"],
    snippet: `let shell = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(20),
    Constraint::Length(7),
    Constraint::Length(3),
]).split(area);`,
  },
  {
    id: "schematic-detail-board",
    title: "Schematic + detail board",
    fit: "Service maps, pipelines, DAGs, health graphs",
    regions: ["schematic min 54", "node detail 34", "event log 10"],
    snippet: `let body = Layout::horizontal([
    Constraint::Min(54),
    Constraint::Length(34),
]).split(area);

let right = Layout::vertical([
    Constraint::Min(12),
    Constraint::Length(10),
]).split(body[1]);`,
  },
];
