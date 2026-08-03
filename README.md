> [!WARNING]
> This app is in active development and not currently intended to be used in a production environment.

# Preaching Partner

Preaching Partner is a mobile application for managing a congregation of Jehovah's Witnesses' territory. It will help track when a map was last worked, which addresses have requested a do not call status, and which groups are working each map.

This app requires the companion server for each congregation.
(https://github.com/SilentFlag/preaching-partner-server)

## Features 

Much of the app is still non-functional, however I am building with several clear goals in mind.
- Easily organise maps so none of the territory is left unworked
- Search maps by name, category, and street names
- Tag addresses which have requested do-not-call status, don't recieve mail, or custom tags
- Check off addresses which have been called so you can go back and do not at homes (timestamps not recorded)

## Why I am building this

My congregation's current solution for managing territory is functional, but could be improved in several areas. I am building to both demonstrate my programming ability and areas where territory management could be improved.

## Tech Stack and Architecture
 
- **Rust**
Rust is the language I have chosen to build this app with. Running the backend logic, connection to server, and database, it provides much of the apps core functionality
- **Tauri**
Tauri is a cross-platform frontend framework to build desktop (Windows, MacOS, Linux) and Mobile (iOS, Android).
 - **HTML, CSS, JS, Svelte**
 Tauri takes advantage of each system's native web renderer for graphics. Using web technologies makes development easy while still being much lighter than other web-based frameworks.
 - **Sqlite**
 Sqlite is the most common database on mobile devices, and was an easy decision to use this.