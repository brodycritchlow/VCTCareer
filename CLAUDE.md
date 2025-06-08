# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

VCTCareer is a full-stack Valorant career simulation platform consisting of:

- **VCTCareerBackend**: Rust/Actix-web API server with PostgreSQL database
- **VCTCareerFrontend**: React SPA built with Rsbuild and Tailwind CSS

## Development Commands

### Backend (Rust)
```bash
# From VCTCareerBackend directory
cargo run                    # Start development server on localhost:8080
cargo test                   # Run all tests including simulation manager tests
cargo build                  # Build project
cargo check                  # Quick compile check without executable
cargo fmt                    # Format Rust code
cargo clippy                 # Run linter

# Run specific examples
cargo run --example simulation_manager_example
cargo run --example modern_simulation_example
```

### Frontend (React)
```bash
# From VCTCareerFrontend directory
npm run dev                  # Start development server with hot reload
npm run build                # Build for production
npm run preview              # Preview production build
npm run check                # Run Biome linter and formatter
npm run format               # Format code with Biome
```

Note: Frontend README indicates using `pnpm` but package.json uses `npm` scripts.

## Architecture Overview

### Backend Architecture
- **Main Server**: `main.rs` sets up Actix-web HTTP server with CORS, PostgreSQL connection pool, and Swagger UI
- **API Endpoints**: Career creation, team queries, offer generation, RR estimation, map selection, and simulation management
- **Core Modules**:
  - `models.rs`: Data structures for CareerInfo, Teams, and enums
  - `db.rs`: Database connection and team queries
  - `offers.rs`: Offer generation system
  - `ranked.rs`: Valorant ranking and RR calculation logic
  - `sim.rs`: Game simulation engine with weapons, agents, and combat mechanics
  - `simulation_manager.rs`: Advanced simulation management with checkpoints, event streaming, and time-travel features

### Simulation Engine Features
The simulation engine (`sim.rs` + `simulation_manager.rs`) provides:
- **Real-time match simulation**: Tick-based Valorant match simulation with weapon mechanics
- **Player statistics tracking**: KDA, damage, headshot percentages, economy
- **Event system**: Comprehensive game event logging (kills, deaths, round events)
- **Advanced controls**: Pause/resume, speed adjustment, checkpoint creation/restoration
- **Event querying**: Filter events by type, player, round, timestamp
- **Live stats**: Real-time scoreboard, economy status, player rankings

### Frontend Architecture
- **Router**: React Router with dynamic navigation based on route (`/` vs `/career/*`)
- **Components**: Modular structure with common components (Navbar) and page-specific components
- **Navigation**: Adaptive navbar that shows different items for landing vs career pages, plus sidebar navigation
- **Styling**: Tailwind CSS with custom configurations and Biome for code formatting

### Key Integrations
- Backend serves API at `localhost:8080` with Swagger docs at `/swagger-ui/`
- Frontend connects to backend APIs for career simulation features
- Database stores team information, offers, and player data
- Simulation manager handles in-memory game state and event history

### Environment Requirements
- Backend requires `DATABASE_URL` environment variable for PostgreSQL connection
- Backend uses `.env` files for configuration (dotenv)

## Testing Strategy

### Backend Tests
- Unit tests in `tests/simulation_manager_tests.rs` for simulation functionality
- Run tests with `cargo test` from VCTCareerBackend directory
- Examples serve as integration test patterns in `examples/` directory

### Code Quality
- Backend: Use `cargo fmt` and `cargo clippy` before committing
- Frontend: Use `npm run check` and `npm run format` (Biome) before committing
- Frontend linting enforces accessibility standards (a11y rules), proper button semantics, and React best practices

## Common Development Workflows

### Frontend Accessibility Standards
- All interactive elements must have proper keyboard navigation support
- Use semantic HTML elements (`<button>` instead of `<div role="button">`)
- Provide `type="button"` for buttons that don't submit forms
- Include `aria-label` attributes for screen readers where appropriate
- Biome enforces these standards automatically via `npm run check`

### Simulation Development
- Use `cargo run --example simulation_manager_example` to test new simulation features
- Simulation state is managed in-memory via `simulation_manager.rs`
- All game events are logged and queryable for debugging and analytics
- Examples in `examples/` directory demonstrate proper simulation API usage