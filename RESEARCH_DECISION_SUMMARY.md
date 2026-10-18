# Research decision and timeline clarification

The completed report recommends retaining the useful Rust reconstruction and moving toward one portable, modular simulation, with original DOS gameplay as a behavioral reference. C/C++ components are permitted where their verified integration cost is lower. A native x86 assembly compatibility route could be faster, but the reported GeneWars bridge has not been independently reproduced as a complete human-playable mission with save/reload.

The report's **6–18+ engineer-months** is a speculative planning range for campaign breadth, product finishing, save reliability and an initial editor/mod surface. It is **not a measured forecast**, a guaranteed completion date or a statement that a playable version cannot arrive sooner.

One engineer-month means roughly one experienced engineer working full-time for a month (about 160 working hours). Effort and calendar time are different; adding people or AI agents does not divide elapsed time linearly. The project currently has valuable implementations and evidence, but no demonstrated dependable completion schedule.

The high-information next steps are the bounded experiments in the report: original-save access, a whole-mission native-bridge demonstration, complete behavior-family implementation, and a real scenario/save/package boundary. These experiments are proposed, not executed. Archiving this work on GitHub does not authorize restarting development workers or implementing a new engine.

