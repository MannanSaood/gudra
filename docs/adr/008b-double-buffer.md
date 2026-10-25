# ADR 008b: Two complete haloed grids

Status: accepted; supplied Chat 08 Arch experimental GPU validation passed.
See the evidence scope in ../handoffs/08-timestep-runtime.md.

Allocate two equal-shaped, distinct haloed grids and one immutable interior RHS.
Every launch reads only current/RHS and writes every next cell; boundary cells
copy current and interior cells evaluate weighted Jacobi. Swap only on successful
completion. This preserves fixed Dirichlet halos across both odd and even batches.

Construction is the only way to obtain a DoubleGrid, so callers cannot mix
sessions, transpose one buffer, change a shape, or import aliased storage. Both
grids and RHS are charged before creating an extra host copy. A scalar-tile
baseline adds a constructor check for <=65,535 blocks per haloed axis.

Interior-only output plus halo repacking would require another kernel or host
transfer per step. Mutable subviews would expand the existing backend safety
boundary. Neither is needed for this baseline. The tradeoff is redundant boundary
copies and poor scalar-block occupancy. Fixed boundaries are explicit; Neumann,
periodic and time-dependent conditions need a separately validated policy/API.
