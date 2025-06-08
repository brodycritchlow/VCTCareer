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
cargo test                   # Run all tests
cargo build                  # Build project
cargo check                  # Quick compile check without executable
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
- **API Endpoints**: Career creation, team queries, offer generation, RR estimation, and map selection
- **Core Modules**:
  - `models.rs`: Data structures for CareerInfo, Teams, and enums
  - `db.rs`: Database connection and team queries
  - `offers.rs`: Offer generation system
  - `ranked.rs`: Valorant ranking and RR calculation logic
  - `sim.rs`: Game simulation engine with weapons, agents, and combat mechanics

### Frontend Architecture
- **Router**: React Router with dynamic navigation based on route (`/` vs `/career/*`)
- **Components**: Modular structure with common components (Navbar) and page-specific components
- **Navigation**: Adaptive navbar that shows different items for landing vs career pages, plus sidebar navigation
- **Styling**: Tailwind CSS with custom configurations

### Key Integrations
- Backend serves API at `localhost:8080` with Swagger docs at `/swagger-ui/`
- Frontend connects to backend APIs for career simulation features
- Database stores team information, offers, and player data

### Environment Requirements
- Backend requires `DATABASE_URL` environment variable for PostgreSQL connection
- Backend uses `.env` files for configuration (dotenv)