---
status: accepted
---

# Sizes follow the window, not the desktop's scale factor

Every interface size is authored in reference units and scales with the window's size alone. The desktop's scale factor does not change how large anything looks relative to the window. That factor is 2 for this game under Xwayland at the owner's 125% GNOME scaling, 1.25 for a native Wayland window, and 1 in the development seat. Because sizes ignore it, a development-seat capture at the owner's display size (3840×2160) shows what the owner's panel shows, whatever the desktop scaling or windowing backend. Agents can then check sizes and legibility before asking the owner (foundation 3, "Required agent-operated playtesting"). The owner accepted this during the 2026-11-18 triage of #50.

## Considered Options

- **Follow the desktop's scale factor,** the usual behaviour for desktop applications: the game's look would change with the GNOME scaling setting. The windowless seat would also need a shared `unfolded-tabletop` change to imitate the owner's factor, and that imitation would go stale whenever the setting changed.

## Consequences

- **A fixed size in logical pixels is a defect.** The piece counts, at a fixed 21, are the known case. Converted sizes are chosen to keep their current size on the owner's display.
- **Desktop scaling cannot enlarge the game's interface.** The window's size and the layout decide how large it is.
