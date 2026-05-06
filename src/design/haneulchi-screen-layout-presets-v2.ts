/** Screen layout presets for Haneulchi v2 visual system. */

export type HaneulchiScreenId =
  | "controlTower"
  | "terminalDeck"
  | "gridSplit"
  | "explorer"
  | "git"
  | "inspector"
  | "preview"
  | "taskBoard";

export type PanelPreset = {
  id: string;
  area: string;
  minW?: number;
  minH?: number;
  defaultW?: number;
  defaultH?: number;
  notes?: string;
};

export type ScreenLayoutPreset = {
  id: HaneulchiScreenId;
  referenceAsset: string;
  viewport: { width: number; height: number };
  shell: "standard" | "large-terminal";
  workspaceTemplate: string;
  panels: PanelPreset[];
  rightRail: boolean;
  bottomPanel: boolean;
};

export const HC_SCREEN_PRESETS: Record<HaneulchiScreenId, ScreenLayoutPreset> = {
  controlTower: {
    id: "controlTower",
    referenceAsset: "assets/control_tower_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs tabs tabs\nkpi kpi kpi\nmap map map\nprojects projects projects\ntimeline timeline timeline`,
    panels: [
      { id: "kpiStrip", area: "kpi", minH: 84, notes: "4 summary cards, equal width" },
      { id: "sessionMap", area: "map", minH: 198, notes: "project graph grouped by project" },
      { id: "activeProjectsTable", area: "projects", minH: 214, notes: "dense table rows, no heavy dividers" },
      { id: "orchestrationTimeline", area: "timeline", minH: 158, notes: "event stream with actor/session columns" },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  terminalDeck: {
    id: "terminalDeck",
    referenceAsset: "assets/terminal_deck_v2.jpeg",
    viewport: { width: 1536, height: 1024 },
    shell: "large-terminal",
    workspaceTemplate: `tabs tabs\nterminal terminal\nlogs logs`,
    panels: [
      { id: "terminalGrid", area: "terminal", minH: 600, notes: "2x2 terminal panes" },
      { id: "bottomLogs", area: "logs", defaultH: 176, minH: 144 },
    ],
    rightRail: true,
    bottomPanel: true,
  },
  gridSplit: {
    id: "gridSplit",
    referenceAsset: "assets/grid_split_v2.jpeg",
    viewport: { width: 1536, height: 1024 },
    shell: "large-terminal",
    workspaceTemplate: `tabs tabs\nterminal explorer\ngit preview`,
    panels: [
      { id: "terminalWithLogs", area: "terminal", minH: 380, notes: "terminal top + compact logs inside same panel" },
      { id: "explorerMini", area: "explorer", minH: 380 },
      { id: "gitMini", area: "git", minH: 380 },
      { id: "previewMini", area: "preview", minH: 380 },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  explorer: {
    id: "explorer",
    referenceAsset: "assets/explorer_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs tabs\ntoolbar toolbar\ntree detail`,
    panels: [
      { id: "explorerToolbar", area: "toolbar", defaultH: 44 },
      { id: "fileTree", area: "tree", defaultW: 446, minW: 360 },
      { id: "fileDetail", area: "detail", minW: 420, notes: "changes list + code viewer + summary tabs" },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  git: {
    id: "git",
    referenceAsset: "assets/git_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs tabs\ntoolbar toolbar\ngraph changes\nprs prs`,
    panels: [
      { id: "gitToolbar", area: "toolbar", defaultH: 48 },
      { id: "commitGraph", area: "graph", defaultW: 470, minW: 360 },
      { id: "changesCommit", area: "changes", minW: 420 },
      { id: "pullRequests", area: "prs", defaultH: 168, minH: 140 },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  inspector: {
    id: "inspector",
    referenceAsset: "assets/inspector_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs tabs tabs\nsubtabs subtabs actions\noverview commands events\nartifacts artifacts environment`,
    panels: [
      { id: "inspectorSubtabs", area: "subtabs", defaultH: 42 },
      { id: "sessionOverview", area: "overview", minH: 320 },
      { id: "recentCommands", area: "commands", minH: 320 },
      { id: "recentEvents", area: "events", minH: 320 },
      { id: "artifacts", area: "artifacts", minH: 220 },
      { id: "environment", area: "environment", minH: 220 },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  preview: {
    id: "preview",
    referenceAsset: "assets/preview_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs tabs\nroutes preview\nroutes events`,
    panels: [
      { id: "previewRoutes", area: "routes", defaultW: 188, minW: 168 },
      { id: "webPreview", area: "preview", minH: 510 },
      { id: "previewEvents", area: "events", defaultH: 148, minH: 128 },
    ],
    rightRail: true,
    bottomPanel: false,
  },
  taskBoard: {
    id: "taskBoard",
    referenceAsset: "assets/task_board_v2.jpeg",
    viewport: { width: 1536, height: 864 },
    shell: "standard",
    workspaceTemplate: `tabs\ntoolbar\ncolumns`,
    panels: [
      { id: "boardToolbar", area: "toolbar", defaultH: 44 },
      { id: "kanbanColumns", area: "columns", notes: "4 visible columns at 1536 width; horizontal scroll allowed" },
    ],
    rightRail: true,
    bottomPanel: false,
  },
};

export const HC_BOARD_COLUMNS = [
  { id: "backlog", label: "Backlog", minW: 232, dot: "muted" },
  { id: "in_progress", label: "In Progress", minW: 232, dot: "primary" },
  { id: "in_review", label: "In Review", minW: 232, dot: "warning" },
  { id: "done", label: "Done", minW: 232, dot: "success" },
] as const;

export const HC_RIGHT_RAIL_SECTIONS = [
  { id: "attention", title: "Attention Center", minH: 300 },
  { id: "review", title: "Review Queue", minH: 190 },
  { id: "recent", title: "Recent Sessions", minH: 190 },
] as const;
