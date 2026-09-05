import { ChevronDown, Code2, Database, FileCode2, FolderTree, Layers3, PackagePlus, Rows3, Search, Table2, Terminal, Workflow } from "lucide-react";
import { useMemo, useState } from "react";
import { CopyButton } from "./CopyButton";
import {
  componentBlueprints,
  folderBrowserRecipes,
  folderRecipes,
  interactionContracts,
  layoutRecipes,
  resourceKits,
  schematicModules,
  schematicRecipes,
  tabRecipes,
  tableRecipes,
  tuiPatterns,
} from "../tuiWorkbenchData";
import type { PatternTab } from "../tuiWorkbenchTypes";

export function TuiPatternsSection() {
  const [activeTab, setActiveTab] = useState<PatternTab>("table");
  const [expanded, setExpanded] = useState(false);
  const [query, setQuery] = useState("");
  const [activeFolder, setActiveFolder] = useState(folderRecipes[0]);
  const [activeFolderBrowserRecipe, setActiveFolderBrowserRecipe] = useState(folderBrowserRecipes[0]);
  const [activeTableRecipe, setActiveTableRecipe] = useState(tableRecipes[0]);
  const [activeSchematicRecipe, setActiveSchematicRecipe] = useState(schematicRecipes[0]);
  const [activeTabRecipe, setActiveTabRecipe] = useState(tabRecipes[0]);
  const [activeContract, setActiveContract] = useState(interactionContracts[0]);
  const [sortKey, setSortKey] = useState<"pattern" | "priority">("priority");

  const filteredPatterns = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const priorityRank = { Core: 0, Next: 1, Lab: 2 };

    return tuiPatterns
      .filter((pattern) =>
        normalized.length === 0
          ? true
          : [pattern.pattern, pattern.module, pattern.useCase, pattern.rustHook, pattern.dotmaxHook, pattern.priority]
              .join(" ")
              .toLowerCase()
              .includes(normalized),
      )
      .sort((a, b) => {
        if (sortKey === "pattern") return a.pattern.localeCompare(b.pattern);
        return priorityRank[a.priority] - priorityRank[b.priority] || a.pattern.localeCompare(b.pattern);
      });
  }, [query, sortKey]);

  const patternResourceIndex: Array<{
    id: PatternTab;
    label: string;
    count: number;
    summary: string;
  }> = [
    {
      id: "table",
      label: "Research table",
      count: tableRecipes.length,
      summary: "Columns, filters, row actions, sparklines, and summaries.",
    },
    {
      id: "folders",
      label: "Folders",
      count: folderRecipes.length + folderBrowserRecipes.length,
      summary: "Tree state, previews, refresh behavior, and split panes.",
    },
    {
      id: "schematic",
      label: "Schematic",
      count: schematicModules.length + schematicRecipes.length,
      summary: "Graph modules, node state, health, routing, and actions.",
    },
    {
      id: "tabs",
      label: "Tabs",
      count: tabRecipes.length,
      summary: "Typed views, scoped commands, retained child state, previews.",
    },
    {
      id: "blueprints",
      label: "Blueprints",
      count: componentBlueprints.length,
      summary: "Component state models and key contracts.",
    },
    {
      id: "kits",
      label: "Resource kits",
      count: resourceKits.length,
      summary: "Dependency bundles, file layouts, and setup commands.",
    },
    {
      id: "contracts",
      label: "Contracts",
      count: interactionContracts.length,
      summary: "Command behaviors and testable state changes.",
    },
    {
      id: "layouts",
      label: "Layouts",
      count: layoutRecipes.length,
      summary: "Ratatui pane compositions for common TUI shells.",
    },
  ];

  return (
    <section id="patterns" className="section patterns-section">
      <div className="patterns-head">
        <div className="section-heading" data-reveal>
          <span className="eyebrow">TUI pattern workbench</span>
          <h2>Reusable interface pieces for Rust terminal software.</h2>
          <p>
            Practical patterns for engineers building dashboards, inspectors, benchmarks, artifact browsers, and operational tools with ratatui plus dotmax previews.
          </p>
        </div>
        <div className="pattern-kpis" aria-label="TUI pattern resource counts">
          <span>
            <strong>{tuiPatterns.length}</strong>
            patterns
          </span>
          <span>
            <strong>{tableRecipes.length}</strong>
            table recipes
          </span>
          <span>
            <strong>{folderRecipes.length}</strong>
            folders
          </span>
          <span>
            <strong>{folderBrowserRecipes.length}</strong>
            browser kits
          </span>
          <span>
            <strong>{schematicModules.length}</strong>
            modules
          </span>
          <span>
            <strong>{schematicRecipes.length}</strong>
            graph kits
          </span>
          <span>
            <strong>{tabRecipes.length}</strong>
            tab maps
          </span>
          <span>
            <strong>{componentBlueprints.length}</strong>
            blueprints
          </span>
          <span>
            <strong>{resourceKits.length}</strong>
            kits
          </span>
          <span>
            <strong>{interactionContracts.length}</strong>
            contracts
          </span>
          <span>
            <strong>{layoutRecipes.length}</strong>
            layouts
          </span>
        </div>
      </div>

      <div className="pattern-resource-index" aria-label="Pattern workbench quick index">
        {patternResourceIndex.map((item) => (
          <button
            className={expanded && activeTab === item.id ? "pattern-resource-card pattern-resource-card-active" : "pattern-resource-card"}
            key={item.id}
            onClick={() => {
              setActiveTab(item.id);
              setExpanded(true);
            }}
            type="button"
            aria-label={`Open ${item.label} resources`}
          >
            <span>{item.label}</span>
            <strong>{item.count}</strong>
            <p>{item.summary}</p>
          </button>
        ))}
      </div>

      {!expanded && (
        <button className="expander-button" type="button" onClick={() => setExpanded(true)}>
          <ChevronDown size={16} />
          Open the pattern workbench
        </button>
      )}

      {expanded && (
        <>
      <div className="pattern-tabs" role="tablist" aria-label="TUI pattern resources">
        <button className={activeTab === "table" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("table")} type="button">
          <Table2 size={16} />
          Research table
        </button>
        <button className={activeTab === "folders" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("folders")} type="button">
          <FolderTree size={16} />
          Folders
        </button>
        <button className={activeTab === "schematic" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("schematic")} type="button">
          <Workflow size={16} />
          Schematic
        </button>
        <button className={activeTab === "tabs" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("tabs")} type="button">
          <Rows3 size={16} />
          Tabs
        </button>
        <button className={activeTab === "blueprints" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("blueprints")} type="button">
          <Code2 size={16} />
          Blueprints
        </button>
        <button className={activeTab === "kits" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("kits")} type="button">
          <PackagePlus size={16} />
          Resource kits
        </button>
        <button className={activeTab === "contracts" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("contracts")} type="button">
          <Terminal size={16} />
          Contracts
        </button>
        <button className={activeTab === "layouts" ? "pattern-tab pattern-tab-active" : "pattern-tab"} onClick={() => setActiveTab("layouts")} type="button">
          <Layers3 size={16} />
          Layouts
        </button>
      </div>

      {activeTab === "table" && (
        <div className="pattern-panel">
          <div className="pattern-toolbar">
            <label className="search-box">
              <Search size={16} />
              <span className="sr-only">Search TUI patterns</span>
              <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search table, folders, schematic, inspector..." />
            </label>
            <div className="pattern-sort" aria-label="Pattern sort controls">
              <button className={sortKey === "priority" ? "pattern-sort-active" : ""} onClick={() => setSortKey("priority")} type="button">
                <Rows3 size={15} />
                Priority
              </button>
              <button className={sortKey === "pattern" ? "pattern-sort-active" : ""} onClick={() => setSortKey("pattern")} type="button">
                <Table2 size={15} />
                Name
              </button>
            </div>
          </div>
          <div className="resource-table-wrap">
            <table className="resource-table">
              <thead>
                <tr>
                  <th>Pattern</th>
                  <th>Ratatui hook</th>
                  <th>Developer use</th>
                  <th>dotmax edge</th>
                  <th>Lane</th>
                </tr>
              </thead>
              <tbody>
                {filteredPatterns.map((pattern) => (
                  <tr key={pattern.pattern}>
                    <td>
                      <strong>{pattern.pattern}</strong>
                      <span>{pattern.rustHook}</span>
                    </td>
                    <td>{pattern.module}</td>
                    <td>{pattern.useCase}</td>
                    <td>{pattern.dotmaxHook}</td>
                    <td>
                      <span className={`priority-pill priority-${pattern.priority.toLowerCase()}`}>{pattern.priority}</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="table-recipe-workbench">
            <div className="table-recipe-list" aria-label="Research table implementation recipes">
              {tableRecipes.map((recipe) => (
                <button
                  className={activeTableRecipe.id === recipe.id ? "table-recipe-row table-recipe-row-active" : "table-recipe-row"}
                  key={recipe.id}
                  onClick={() => setActiveTableRecipe(recipe)}
                  type="button"
                >
                  <span>{recipe.id}</span>
                  <strong>{recipe.title}</strong>
                  <code>{recipe.column}</code>
                </button>
              ))}
            </div>
            <div className="table-recipe-detail">
              <div className="table-recipe-head">
                <div>
                  <span>{activeTableRecipe.column}</span>
                  <h3>{activeTableRecipe.title}</h3>
                  <p>{activeTableRecipe.fit}</p>
                </div>
                <CopyButton value={activeTableRecipe.snippet} />
              </div>
              <div className="table-recipe-preview" aria-label={`${activeTableRecipe.title} terminal preview`}>
                {activeTableRecipe.preview.map((line) => (
                  <span key={line}>{line}</span>
                ))}
              </div>
              <div className="table-recipe-grid">
                <div>
                  <strong>State</strong>
                  {activeTableRecipe.state.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
                <div>
                  <strong>Controls</strong>
                  {activeTableRecipe.filters.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
              </div>
              <pre className="table-recipe-code">
                <code>{activeTableRecipe.snippet}</code>
              </pre>
            </div>
          </div>
        </div>
      )}

      {activeTab === "folders" && (
        <div className="pattern-panel pattern-folder-layout">
          <div className="folder-tree-panel" aria-label="Rust TUI folder recipes">
            {folderRecipes.map((folder) => (
              <button className={activeFolder.path === folder.path ? "folder-node folder-node-active" : "folder-node"} key={folder.path} onClick={() => setActiveFolder(folder)} type="button">
                <FileCode2 size={15} />
                <span>{folder.path}</span>
              </button>
            ))}
          </div>
          <div className="folder-detail-panel">
            <div className="folder-detail-top">
              <FolderTree size={20} />
              <div>
                <span>{activeFolder.label}</span>
                <strong>{activeFolder.path}</strong>
              </div>
            </div>
            <p>{activeFolder.summary}</p>
            <div className="folder-recipe-box">
              <span>Implementation note</span>
              <code>{activeFolder.recipe}</code>
            </div>
            <div className="folder-browser-workbench">
              <div className="folder-browser-list" aria-label="Folder browser implementation recipes">
                {folderBrowserRecipes.map((recipe) => (
                  <button
                    className={activeFolderBrowserRecipe.id === recipe.id ? "folder-browser-row folder-browser-row-active" : "folder-browser-row"}
                    key={recipe.id}
                    onClick={() => setActiveFolderBrowserRecipe(recipe)}
                    type="button"
                  >
                    <span>{recipe.id}</span>
                    <strong>{recipe.title}</strong>
                  </button>
                ))}
              </div>
              <div className="folder-browser-detail">
                <div className="folder-browser-head">
                  <div>
                    <span>artifact browser recipe</span>
                    <h3>{activeFolderBrowserRecipe.title}</h3>
                    <p>{activeFolderBrowserRecipe.fit}</p>
                  </div>
                  <CopyButton value={activeFolderBrowserRecipe.snippet} />
                </div>
                <div className="folder-browser-preview" aria-label={`${activeFolderBrowserRecipe.title} terminal preview`}>
                  {activeFolderBrowserRecipe.preview.map((line) => (
                    <span key={line}>{line}</span>
                  ))}
                </div>
                <div className="folder-browser-grid">
                  <div>
                    <strong>State</strong>
                    {activeFolderBrowserRecipe.state.map((item) => (
                      <code key={item}>{item}</code>
                    ))}
                  </div>
                  <div>
                    <strong>Events</strong>
                    {activeFolderBrowserRecipe.events.map((item) => (
                      <code key={item}>{item}</code>
                    ))}
                  </div>
                </div>
                <pre className="folder-browser-code">
                  <code>{activeFolderBrowserRecipe.snippet}</code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === "schematic" && (
        <div className="pattern-panel schematic-panel">
          <div className="schematic-grid" aria-label="Modular TUI schematic">
            {schematicModules.map(([title, detail], index) => (
              <div className="schematic-node" key={title}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <strong>{title}</strong>
                <p>{detail}</p>
              </div>
            ))}
          </div>
          <div className="schematic-notes">
            <Database size={18} />
            <p>
              Keep the graph explicit: input updates state, state selects layouts, layouts render widgets, and dotmax owns dense visual previews that ratatui widgets can embed.
            </p>
          </div>
          <div className="schematic-recipe-workbench">
            <div className="schematic-recipe-list" aria-label="Schematic graph implementation recipes">
              {schematicRecipes.map((recipe) => (
                <button
                  className={activeSchematicRecipe.id === recipe.id ? "schematic-recipe-row schematic-recipe-row-active" : "schematic-recipe-row"}
                  key={recipe.id}
                  onClick={() => setActiveSchematicRecipe(recipe)}
                  type="button"
                >
                  <span>{recipe.id}</span>
                  <strong>{recipe.title}</strong>
                </button>
              ))}
            </div>
            <div className="schematic-recipe-detail">
              <div className="schematic-recipe-head">
                <div>
                  <span>graph state recipe</span>
                  <h3>{activeSchematicRecipe.title}</h3>
                  <p>{activeSchematicRecipe.fit}</p>
                </div>
                <CopyButton value={activeSchematicRecipe.snippet} />
              </div>
              <div className="schematic-recipe-preview" aria-label={`${activeSchematicRecipe.title} terminal preview`}>
                {activeSchematicRecipe.preview.map((line) => (
                  <span key={line}>{line}</span>
                ))}
              </div>
              <div className="schematic-recipe-grid">
                <div>
                  <strong>Node model</strong>
                  {activeSchematicRecipe.nodes.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
                <div>
                  <strong>Signals</strong>
                  {activeSchematicRecipe.signals.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
              </div>
              <pre className="schematic-recipe-code">
                <code>{activeSchematicRecipe.snippet}</code>
              </pre>
            </div>
          </div>
        </div>
      )}

      {activeTab === "tabs" && (
        <div className="pattern-panel tab-recipe-panel">
          <div className="tab-recipe-list" aria-label="TUI tab recipes">
            {tabRecipes.map((recipe) => (
              <button
                className={activeTabRecipe.id === recipe.id ? "tab-recipe-row tab-recipe-row-active" : "tab-recipe-row"}
                key={recipe.id}
                onClick={() => setActiveTabRecipe(recipe)}
                type="button"
              >
                <span>{recipe.id}</span>
                <strong>{recipe.title}</strong>
                <code>{recipe.keymap}</code>
              </button>
            ))}
          </div>
          <div className="tab-recipe-detail">
            <div className="tab-recipe-head">
              <div>
                <span>tab state recipe</span>
                <h3>{activeTabRecipe.title}</h3>
                <p>{activeTabRecipe.fit}</p>
              </div>
              <CopyButton value={activeTabRecipe.snippet} />
            </div>
            <div className="tab-strip-preview" aria-label={`${activeTabRecipe.title} tab strip preview`}>
              {activeTabRecipe.views.map((view, index) => (
                <span className={index === 0 ? "tab-strip-active" : ""} key={view}>
                  {view}
                </span>
              ))}
            </div>
            <div className="tab-state-grid">
              <div>
                <strong>State to keep</strong>
                {activeTabRecipe.state.map((item) => (
                  <code key={item}>{item}</code>
                ))}
              </div>
              <div>
                <strong>Key contract</strong>
                <code>{activeTabRecipe.keymap}</code>
                <code>tab switch must not reset child views</code>
                <code>footer hints derive from active view</code>
              </div>
            </div>
            <pre className="tab-recipe-code">
              <code>{activeTabRecipe.snippet}</code>
            </pre>
          </div>
        </div>
      )}

      {activeTab === "blueprints" && (
        <div className="pattern-panel blueprint-panel">
          {componentBlueprints.map((blueprint) => (
            <article className="blueprint-card" key={blueprint.id}>
              <div className="blueprint-card-head">
                <div>
                  <span>{blueprint.id}</span>
                  <h3>{blueprint.title}</h3>
                </div>
                <CopyButton value={blueprint.code} />
              </div>
              <p>{blueprint.problem}</p>
              <div className="blueprint-grid">
                <div>
                  <strong>State</strong>
                  {blueprint.state.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
                <div>
                  <strong>Keys</strong>
                  {blueprint.keys.map((item) => (
                    <code key={item}>{item}</code>
                  ))}
                </div>
              </div>
              <pre className="blueprint-code">
                <code>{blueprint.code}</code>
              </pre>
            </article>
          ))}
        </div>
      )}

      {activeTab === "kits" && (
        <div className="pattern-panel kit-panel">
          {resourceKits.map((kit) => (
            <article className="kit-card" key={kit.id}>
              <div className="kit-card-main">
                <div className="kit-card-head">
                  <span>{kit.id}</span>
                  <h3>{kit.title}</h3>
                  <p>{kit.fit}</p>
                </div>
                <div className="kit-command">
                  <code>{kit.command}</code>
                  <CopyButton value={kit.command} />
                </div>
                <pre className="kit-code">
                  <code>{kit.code}</code>
                </pre>
              </div>
              <div className="kit-card-side">
                <div>
                  <strong>Dependencies</strong>
                  {kit.dependencies.map((dependency) => (
                    <span key={dependency}>{dependency}</span>
                  ))}
                </div>
                <div>
                  <strong>Files</strong>
                  {kit.files.map((file) => (
                    <span key={file}>{file}</span>
                  ))}
                </div>
                <div>
                  <strong>Checklist</strong>
                  {kit.checklist.map((item) => (
                    <span key={item}>{item}</span>
                  ))}
                </div>
              </div>
            </article>
          ))}
        </div>
      )}

      {activeTab === "contracts" && (
        <div className="pattern-panel contract-panel">
          <div className="contract-list" aria-label="Interaction command contracts">
            {interactionContracts.map((contract) => (
              <button
                className={activeContract.id === contract.id ? "contract-row contract-row-active" : "contract-row"}
                key={contract.id}
                onClick={() => setActiveContract(contract)}
                type="button"
              >
                <span>{contract.id}</span>
                <strong>{contract.scope}</strong>
                <code>{contract.keys}</code>
              </button>
            ))}
          </div>
          <div className="contract-detail">
            <div className="contract-detail-head">
              <div>
                <span>{activeContract.scope}</span>
                <h3>{activeContract.id}</h3>
              </div>
              <CopyButton value={activeContract.snippet} />
            </div>
            <dl>
              <div>
                <dt>Keys</dt>
                <dd>{activeContract.keys}</dd>
              </div>
              <div>
                <dt>State change</dt>
                <dd>{activeContract.stateChange}</dd>
              </div>
              <div>
                <dt>Test contract</dt>
                <dd>{activeContract.test}</dd>
              </div>
            </dl>
            <pre className="contract-snippet">
              <code>{activeContract.snippet}</code>
            </pre>
          </div>
        </div>
      )}

      {activeTab === "layouts" && (
        <div className="pattern-panel layout-panel">
          {layoutRecipes.map((layout) => (
            <article className="layout-card" key={layout.id}>
              <div className="layout-wireframe" aria-label={`${layout.title} wireframe`}>
                {layout.regions.map((region, index) => (
                  <span key={region} style={{ minHeight: `${index === 1 ? 6 : 2.2}rem` }}>
                    {region}
                  </span>
                ))}
              </div>
              <div className="layout-card-body">
                <span>{layout.id}</span>
                <h3>{layout.title}</h3>
                <p>{layout.fit}</p>
                <div className="layout-region-row">
                  {layout.regions.map((region) => (
                    <code key={region}>{region}</code>
                  ))}
                </div>
                <div className="layout-code-head">
                  <strong>ratatui layout</strong>
                  <CopyButton value={layout.snippet} />
                </div>
                <pre className="layout-code">
                  <code>{layout.snippet}</code>
                </pre>
              </div>
            </article>
          ))}
        </div>
      )}
        </>
      )}
    </section>
  );
}
