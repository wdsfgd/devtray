# Use GTK3 for System Tray UI

We decided to build DevTray using Python with GTK3 (`PyGObject`) and `AppIndicator3` instead of GTK4 or PySide6.

While GTK4 is newer, it has completely removed system tray support (`Gtk.StatusIcon`) to align with modern GNOME design guidelines. The Linux ecosystem standard for system trays is `libayatana-appindicator3`, which relies on GTK3. Loading both GTK3 (for the tray) and GTK4 (for the main window) in the same Python process causes fatal GLib conflicts. We also chose this over PySide6 because GTK3 leverages system-shared libraries, resulting in a negligible RAM and disk footprint compared to bundling Qt.
