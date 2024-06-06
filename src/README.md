# Rusty CV Creator

## Dependencies

`.env`

must define the database variable, must match the one we have in the ` ~/.config/rusty-cv-creator/rusty-cv-config.ini`

```sh
DATABASE_URL=sqlite://$HOME/.config/rusty-cv-creator/applications.db
```


## TODO

- Make sure that we source the right config.ini file from `~/.config/rusty-cv-generator/config.ini`
- Make sure that we source the right database file from `~/.config/rusty-cv-generator/database.db`
- Implement `Update`, `Delete`, `Select` and `List`.
  - Implement `List` with a filter.
