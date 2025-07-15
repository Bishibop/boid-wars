# Boid Wars Documentation

## Quick Links

- [🚀 Getting Started](../README.md)
- [🗺️ Roadmap](../ROADMAP.md)
- [🤝 Contributing](../CONTRIBUTING.md)
- [⚡ Quick Reference](development/QUICK_REFERENCE.md)

## Technical Documentation

### Architecture & Design
- [System Architecture](../ARCHITECTURE.md) - **Living document** - Updated after each milestone
- [Architecture Review](technical/ARCHITECTURE_REVIEW.md) - Review notes and decisions
- [Tech Stack Research](technical/TECH_STACK_RESEARCH.md) - Technology selection rationale
- [Tech Stack Validation](technical/tech_stack_integration_validation.md) - Integration testing plan
- [Asset Strategy](technical/ASSET_STRATEGY.md) - Asset pipeline and optimization

### Development
- [Coding Standards](development/CODING_STANDARDS.md) - Style guide and best practices
- [Development Guide](development.md) - Setup and workflow
- [Development Tools](development/development_tools.md) - Tool configuration
- [Quick Reference](development/QUICK_REFERENCE.md) - Common commands and tips
- [Security Checklist](development/security-checklist.md) - Security considerations

### Game Design
- [AI Behaviors](AI_BEHAVIORS.md) - Boid AI system design
- [Architecture Decisions](architecture-decisions.md) - ADRs and design choices

### Operations
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Documentation Structure

```
docs/
├── README.md                 # This file
├── technical/               # Architecture and technical design
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE_REVIEW.md
│   ├── TECH_STACK_RESEARCH.md
│   ├── tech_stack_integration_validation.md
│   └── ASSET_STRATEGY.md
├── development/             # Development guides and standards
│   ├── CODING_STANDARDS.md
│   ├── development_tools.md
│   ├── QUICK_REFERENCE.md
│   └── security-checklist.md
├── AI_BEHAVIORS.md          # Game-specific documentation
├── architecture-decisions.md
├── development.md
└── troubleshooting.md
```