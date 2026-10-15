# Krell next construction sites

Current captured map: callback054 frame1017 slot5 after. Camera target [1536,15360] gives camera cells [248,56]. Both Distributor3 and Genepod0 use footprint3, a five-by-five sampling region with a three-by-three interior.

Practical candidates are recorded in the adjacent JSON, including forward screen projection and independently translated inverse-picker coordinates. Prefer Distributor at screen **[224,299]**, snapped world **[1919,16511]** (cell7,64), and a later Genepod at **[448,226]**, snapped world **[1919,14719]** (cell7,57). Their footprints are separated. Confirm the ordinary dispatcher result as part of Root's next integration run; this read-only scan is not gameplay acceptance.

Every listed candidate passes the existing footprint terrain tests: sampled nonboundary cells have terrain class greater than1; interior flags exclude1 and terrain codes20..22. All nine interior entity-list heads are empty, a conservative sufficient condition for the source8B340 query returning no obstacle. No terrain, funds, actors, or source resources were changed. The rejected Distributor cell11,62 contains water in the required footprint; moving the Architect there cannot fix that site.

Resource/occupancy changes during subsequent callbacks can invalidate a candidate. Re-evaluate at actual placement. Completed-building collision and next command sequencing remain ordinary runtime responsibilities. The Genepod candidate is a future location, not a claim that its prerequisites are currently satisfied.
