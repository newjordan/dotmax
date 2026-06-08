export type TuiPattern = {
  pattern: string;
  module: string;
  useCase: string;
  rustHook: string;
  dotmaxHook: string;
  priority: "Core" | "Next" | "Lab";
};

export type TableRecipe = {
  id: string;
  title: string;
  fit: string;
  column: string;
  state: string[];
  filters: string[];
  snippet: string;
  preview: string[];
};

export type FolderBrowserRecipe = {
  id: string;
  title: string;
  fit: string;
  state: string[];
  events: string[];
  preview: string[];
  snippet: string;
};

export type SchematicRecipe = {
  id: string;
  title: string;
  fit: string;
  nodes: string[];
  signals: string[];
  preview: string[];
  snippet: string;
};

export type PatternTab = "table" | "folders" | "schematic" | "tabs" | "blueprints" | "kits" | "contracts" | "layouts";

export type ComponentBlueprint = {
  id: string;
  title: string;
  problem: string;
  state: string[];
  keys: string[];
  code: string;
};

export type ResourceKit = {
  id: string;
  title: string;
  fit: string;
  dependencies: string[];
  files: string[];
  checklist: string[];
  command: string;
  code: string;
};

export type TabRecipe = {
  id: string;
  title: string;
  fit: string;
  views: string[];
  state: string[];
  keymap: string;
  snippet: string;
};

export type InteractionContract = {
  id: string;
  scope: string;
  keys: string;
  stateChange: string;
  test: string;
  snippet: string;
};

export type LayoutRecipe = {
  id: string;
  title: string;
  fit: string;
  regions: string[];
  snippet: string;
};
