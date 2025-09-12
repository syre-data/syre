[x] Initial start up not signing in after sign up
[x] project manifest not created automatically
[x] drag and drop scripts broken
[x] Toggle autorun from container not update checkmark in properties
[x] Shift + wheel scroll horizontal
[x] Add association editor to container bulk editor
[x] Is union the intuitive operation for `tags` and `metadata` in bulk editing
[x] Metadata map type
[x] Sort metadata in container preview in canvas, update display
[x] Add Metadata popout doesn't close for assets
[x] Bulk edit asset metadata number update not saved (check container and mixed, too)
[x] Messages
[x] Visual feedback during analysis
[x] Renaming root container
[x] Analyses already in analysis root
[x] List display metadata preview
[x] Runner/User settings
[x] Recursive `.syreignore`, use `ignore::WalkDirBuilder`?
[x] Viktor errors
    [x] Report error in analysis
    [x] `syre.dev_mode` not working
    [x] Increase debounce for inputs
    [x] Typing decimal in number metadata
    [x] Bulk editing defocuses on save
    [x] Tracing logging
    [x] Large tree duplication across drives on Windows
    [x] Duplication metadata not being copied
    [x] List metadata content being removed
    [x] Crashes on first open
[x] Copy-paste containers with `.syre` folders, by default reassign ids, in future can prompt to keep
    - Verify there are no repeated resource ids
[x] Remove {`Duplicate`, `Trash`} from root container's context menu
[x] Add child button in canvas is flakey
[x] Hide subtree
[x] Container connectors
[x] Asset icons
[x] Collapse/expand canvas graph when toggling container visibility
[x] Changing name of duplicated folder crashes
[x] Change selection to individual signals
[x] Zoom centered
[x] Set hidden attribute on `.syre` folders on Windows [see `local::common::fs::TempDir`]
[x] Set canvas viewbox when centering container (double click from layers bar)
    - After center, container from layers nav, jumps on drag
[x] Recenter window when toggling container visibility
[x] List data
    - Sometimes gets deleted
    - Sometimes last character will get removed
[x] Save properites on field blur and editor close
[x] Analyze from container
[x] Cancel analysis
[x] Analysis status
[x] Save properties changes on close editor if dirty (See [https://docs.rs/leptos_reactive/latest/leptos_reactive/fn.on_cleanup.html])
[x] Ignore analysis errors
[x] Turn off analyses
[x] Toggle autorun from context menu
[x] Login/out crashes doesn't redirect
[x] Project specific Python/R paths
[x] Specify max tasks, project specific
[x] Flags
    - Store in `.syre`
    - Check if resource is flagged from script
[x] Duplicate project
[x] Default asset `type` on drag and drop
[x] R lang bindings
[x] UTF8 character display
[x] Long asset names push controls to right
    - Truncate right, middle?
[x] Drag-drop large folder onto canvas failed
[x] Bulk editing errors
[x] Remove asset with missing file does not work
[x] Errors occur when removing selected resources
[x] Visual feedback for drag-drop script (cursor, dragover, dragdrop)
[x] Upgrade to tailwind 4
[x] Fix dashboard styling
[x] Lock files and folders on analysis
[x] Update analysis associations from container properties editor breaks 
[x] Search
    - Metadata search column
[x] Drag-drop multiple analysis files that are already in `analysis` folder only adds one
[x] Turn off surreal logs
[x] Autoupdate server
[x] About page with version
[x] Select download channel from settings, w/ no update option
[x] Turn off autorun after script runs successfully
[x] `Div#height`s don't adjust so scroll bars don't work
[x] Database view
[x] In `workspace_db`, column width for pinnable columns not set until moved by user
[x] Resizing column in desktop db workspace below minimum causes odd behavior
[x] `spawn_local` tasks get run everytime user returns to the `Dashboard`
[x] In language binding, parse string for quantity metadata
[] Spreadsheet analysis
[] db workspace
    - Load in background
    - Include containers
    - Open assets
    - Editor - Single and Bulk
    - `Shift + click` for bulk select
    - Select all
[] [internal] Move workspace nav icons into respective workspace for consistancy. (circular dependency?)
[] [internal] Create `ProjectBar` component for reuse in workspaces.
[] Analyze only selected containers, i.e. not their subtrees
[] Extract global UI functions into `Action`s wrapped in new types and passed down
[] Initial loading on start up is slow
    - Likely caused by initial wasm load
[] Make fast
    - Move canvas position calculations to backend for multithreading?
[] Drag drop a container onto another for duplication
    - Duplicate into different location
[] Killing analysis isn't working
[] Flags popout for containers in canvas is hidden
[] Portable executable doesn't work
[] Autoupdater fails for debug builds (See Issue https://github.com/tauri-apps/plugins-workspace/issues/2697)
[] Program hangs on last analysis when running many (flaky)
[] Changing metadata of multiple assets causes crash (flaky?)
[] Second key press in `key` field when adding new metadata causes flicker
[] Refresh should only reload canvas content, not reload entire page
[] Turn off autorun for container or all containers in selection
[] Splashscreeen + download/install page
[] Initializing large projects will fail while initializing folders
[] Sort project analyses
[] Add analysis association box appears off window if editor is full
[] Show projects with missing folders
[] Error if attempting to create a new project in a folder that is not empty
[] Focus `key` input on add metadata
[] Focus input on "Add child" from desktop
[] Add Analyses popout detail submit on enter, make `form` for auto submit on Enter, check others for same (quantity)
[] Layers bar Assets collapses when assets are changed
[] Set `canvas` `viewbox` to match window's aspect ratio, update on resize
[] Drag-drop/copy-paste metadata
[] Analysis results view in desktop
    - Shows results of the most recent analysis run, whether scripts were successful or what errors occurred
    - Ability to save
[] Recent tabs
[] Convert project versioning
[] Project converter
[] Project loader by version
[] Layers nav: container with error
[] Add event for ignore files to `fs_watcher` and update state in `database`
[] Add titles to context menus in canvas
[] Allow errors to be sent from tauri to frontend via updates
[] Delete project folder
[] Allow autoremove messages
[] Drag-drop data that is in a container but not tracked as an Asset
[] Project daemon will respond to lock files (e.g. `~.myfile.docx`)
[] File change watcher for desktop for settings
[] In language bindings change `chdir` to `context` and allow {`Analysis`, `Container`, `None`}
[] Send errors from `local/database` (See `event`)
[] Message autoclose on timeout
[] Reorganize canvas separating root container from rest [internal]
[] Adjust container size in canvas (drag?)
[] Console for analysis output
[] Optimistic updates for properties updates
[] Analysis association wizard: add associations to container based on property/metadata/level
[] `Notes` for resources
[] Ensure correct paths in `flags.json` otherwise show as corrupted.
[] Use container queries to adjust font size for titles
    - https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries
[] Allow editing metadata of existing assets and containers from analysis
    - Track how metadata was edited?
    - Only allow new metadata to be added? i.e. Can not overwrite metadata set by hand
    - Allow metadata to be marked as editable?
    - `blame.json` file
[] Mac and Linux compatibility
[] Testing framework
[] Random folder/project generation script
[] Simulator
[] If an ananlysis is run and the user refreshes the page they can trigger a second analysis because the current analysis state is lost
[] Project repair wizard
[]   - If untracked resources are in a project ask the user how to handle them
[] Allow users not to update app
[] Preview number of analyses that will run
[] Warn user if language bindings do not correspond to desktop version
[] Add badge to flags tab in properties editor bar
[] Desktop does not respond to project/data_root manual changes
[] Hide container data from ancestors
    - Have "bad" data that you don't want included in further analysis so can mark it to be ignored
[] Rotate graph view 90 degrees
[] Updating analysis script file name not updating in desktop
[] Close settings with `Esc`
[] Close desktop database workspace filter bar with `Esc`
[] Include project daemon with language bindings as an extra feature only or source it from the desktop
[] Ensure tags are being deduplicated
[] Context menu to open an asset's file from db workspace
    
? Windows: Parentheses in file paths kills runner (try single quotes)?
? Show missing project in dashboard, with message?
? Unique file name adds suffix if name is already unique (drag-drop)?

# Version converter
[] Read version from each project