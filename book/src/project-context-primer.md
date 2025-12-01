# Project Context Primer

This book focuses on the Nomos Testing Framework. It assumes familiarity with
the Nomos architecture, but for completeness, here is a short primer.

- **Nomos** is a modular blockchain protocol composed of validators, executors,
  and a data-availability (DA) subsystem.
- **Validators** participate in consensus and produce blocks.
- **Executors** are validators with the DA dispersal service enabled. They perform
  all validator functions plus submit blob data to the DA network.
- **Data Availability (DA)** ensures that blob data submitted via channel operations
  in transactions is published and retrievable by the network.

These roles interact tightly, which is why meaningful testing must be performed
in multi-node environments that include real networking, timing, and DA
interaction.
