## ADDED Requirements

### Requirement: Presentation mode selection
The presentations screen SHALL offer the presentation modes the reference defines — the built-in web presenter and the external presentation application bridge — as a single-choice control, and SHALL indicate which is active. Controls and panels that apply only to one mode SHALL appear only in that mode.

#### Scenario: Switching mode
- **WHEN** the user selects a mode
- **THEN** that mode becomes active and is indicated
- **AND** panels specific to the other mode are not shown

#### Scenario: Loaded deck is indicated
- **WHEN** a deck is loaded
- **THEN** the screen shows an active indication alongside the mode

### Requirement: Slide transport
The screen SHALL provide transport controls to move to the first, previous, next and last slide, to stop or unload the presentation, and — in the external application mode — to start it. It SHALL display the current slide position against the total, the loaded deck's name, and a status describing the current mode and readiness. Every transport control SHALL carry an accessible name.

#### Scenario: Moving through slides
- **WHEN** the user activates a transport control
- **THEN** the presentation moves accordingly and the displayed position updates

#### Scenario: Position bounds
- **WHEN** the presentation is on the first or last slide
- **THEN** moving beyond the deck's bounds does not occur

#### Scenario: Stopping
- **WHEN** the user stops or unloads
- **THEN** the deck is unloaded, the position resets, and the status reflects that nothing is loaded

#### Scenario: No deck loaded
- **WHEN** no deck is loaded
- **THEN** the status says so and transport controls that require a deck are unavailable

### Requirement: Deck search and opening
The screen SHALL let the user search the available presentation files and open one directly, or add it to the preload queue. Results SHALL be limited to a readable number and SHALL show each file's name and the folder it belongs to.

#### Scenario: Searching for a deck
- **WHEN** the user enters a search term
- **THEN** matching presentation files are listed with their names and folders

#### Scenario: Opening a deck
- **WHEN** the user opens a result
- **THEN** that deck is loaded, the position resets to the first slide, and the status updates

#### Scenario: No results
- **WHEN** a search matches nothing
- **THEN** an empty state is shown

### Requirement: Preload queue
The screen SHALL provide a fixed number of preload slots, showing which are filled and which are empty, allowing a queued deck to be opened or cleared from its slot, and indicating which queued deck is currently loaded. When every slot is filled, queueing SHALL be unavailable rather than silently discarding.

#### Scenario: Queueing a deck
- **WHEN** the user queues a deck and a slot is free
- **THEN** it occupies the next free slot and the filled count updates

#### Scenario: Queue full
- **WHEN** every slot is filled
- **THEN** the queue action is unavailable and the state is communicated

#### Scenario: Clearing a slot
- **WHEN** the user clears a slot
- **THEN** it becomes empty and the filled count updates

#### Scenario: Loaded deck indicated in the queue
- **WHEN** a queued deck is the loaded one
- **THEN** its slot is distinguished from the other filled slots

### Requirement: Presenter distribution
In web presenter mode the screen SHALL show the address presenters connect to, with a means of copying it that confirms the copy, and a preview of the currently displayed slide with its position. When nothing is loaded the preview SHALL say it is waiting.

#### Scenario: Copying the address
- **WHEN** the user copies the presenter address
- **THEN** it is placed on the clipboard and the action confirms, reverting after a short interval

#### Scenario: Slide preview
- **WHEN** a deck is loaded
- **THEN** the current slide's content and its position are previewed

#### Scenario: Waiting state
- **WHEN** no deck is loaded
- **THEN** the preview indicates it is waiting for a presentation

### Requirement: Connected client visibility
The screen SHALL list the clients currently connected to the presenter, showing for each a label, the client software or device, how long it has been connected and how recently it responded. The total connected count SHALL be shown.

#### Scenario: Clients listed
- **WHEN** clients are connected
- **THEN** each is listed with its label, client description, connection age and last response age
- **AND** the total is shown

#### Scenario: Client connects or disconnects
- **WHEN** a client connects or disconnects
- **THEN** the list and total update without a manual refresh

#### Scenario: No clients
- **WHEN** no client is connected
- **THEN** an empty state is shown rather than an empty list

### Requirement: Presentation folder configuration
The screen SHALL provide access to the configured presentation folders, showing each folder's name and location, and allowing the configuration to be changed and persisted.

#### Scenario: Viewing folders
- **WHEN** the user opens folder configuration
- **THEN** each configured folder's name and location are shown

#### Scenario: Changing folders
- **WHEN** the user changes the folder configuration
- **THEN** the change is persisted through the core and reflected in deck search

### Requirement: Presentations run on live data
Decks, folders, slides and connected clients SHALL come from the core through the shared client package, with transport commands and client changes carried over the core's realtime channel. The screen SHALL NOT contain a sample file library, sample slides or sample clients.

#### Scenario: No fixture content
- **WHEN** the presentations screen is inspected
- **THEN** no hard-coded file, slide or client is used as a data source

#### Scenario: Realtime state
- **WHEN** the presentation position or client set changes at the core
- **THEN** the screen reflects it without a manual refresh
