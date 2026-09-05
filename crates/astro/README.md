# teistro-astro

The astronomy layer of the Teistro SDK: everything above raw positions.
This seed holds the shared boundary solver (`solve`), the one root finder
every event search in the SDK uses (sankrantis, ingresses, rise and set,
stations). The IAU routines, frame completion, the ayanamsha catalogue,
house systems and the rise and set solver arrive with the ephemeris port
(`docs/02-architecture/01-module-catalog.md`).
