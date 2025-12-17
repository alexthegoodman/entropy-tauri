# Entropy Chat

Entropy Chat (the tauri app with Leptos on the frontend) is the WASM app for entropy-engine which allows users to create games via AI chat.

Listed below are the chat actions (tool calls / function use) which enables the workflow.

## Existing Chat Actions

- Transform Object (Models, Lights) (translate, scale, rotate)

## Needed Chat Actions (ready to be added to chat right away)

- Configure the water + add / remove
- Configure the grass + add / remove
- Configure the trees + add / remove
- Configure, add, remove point lights and directional light
- Configure the Player Character
- Add NPC with Chosen associated Model
- Add Game Behaviors to NPC's
- Manage physics settings of Models
- Configure game controls
- Control camera type and configure it
- Import Object (TBD: optionally specify LOD options)
- Import heightmap landscape with PBR textures (need good way of loading what can be over a dozen image files)

## Planned Chat Actions (once features are ready in engine)

- Add Game Mechanics to Player Character (inventory)
- Configure skybox
- Associate animations of model with NPC state machine
- Setup state machines with variables for game data
- Manage in-game UI and menus
- Manage audio

## Action Schema

- All fields are optional and have good defaults
- Specify an object
- Specify an action
- Specify parameters special to that action