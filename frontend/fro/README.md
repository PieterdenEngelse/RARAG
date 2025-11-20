# Development

Your new jumpstart project includes basic organization with an organized `assets` folder and a `components` folder.
If you chose to develop with the router feature, you will also have a `views` folder.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # The entrypoint for the app. It also defines the routes for the app.
│  ├─ components/
│  │  ├─ mod.rs # Defines the components module
│  │  ├─ hero.rs # The Hero component for use in the home page
│  │  ├─ echo.rs # The echo component uses server functions to communicate with the server
│  ├─ views/ # The views each route will render in the app.
│  │  ├─ mod.rs # Defines the module for the views route and re-exports the components for each route
│  │  ├─ blog.rs # The component that will render at the /blog/:id route
│  │  ├─ home.rs # The component that will render at the / route
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve --platform web
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

---

## Tailwind guardrails (avoid Node OOM)

- Tailwind content scanning has been scoped in `tailwind.config.js` to only:
  - `./src/**/*.{rs,html,js,jsx,ts,tsx}`
  - `./public/index.html`
  - `./assets/**/*.html`
- Use the provided npm scripts:
  - `npm run css:build` – one-off build
  - `npm run css:watch` – watcher
- If your machine has tight RAM, increase Node’s heap for Tailwind:
  - Temporary: `export NODE_OPTIONS="--max-old-space-size=4096"`
  - Per command: `NODE_OPTIONS="--max-old-space-size=4096" npm run css:watch`

Recommended dev flow:

1. Build CSS once:
   ```bash
   cd frontend/fro
   npm run css:build
   ```
2. Start Dioxus dev server in a separate terminal:
   ```bash
   cd frontend/fro
   dx serve --platform web
   ```
3. If you need live CSS:
   ```bash
   cd frontend/fro
   NODE_OPTIONS="--max-old-space-size=4096" npm run css:watch
   ```
