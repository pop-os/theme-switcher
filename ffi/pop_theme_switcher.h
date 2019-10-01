#include <gtk/gtk.h>

typedef struct { } PopThemeSwitcher;

PopThemeSwitcher *pop_theme_switcher_new (void);

void pop_theme_switcher_grab_focus (const PopThemeSwitcher *self);

GtkWidget *pop_theme_switcher_widget (const PopThemeSwitcher *self);

void pop_theme_switcher_free (PopThemeSwitcher *self);
