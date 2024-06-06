# Rusty CV Creator

## Dependencies

`.env`

must define the database variable, must match the one we have in the ` ~/.config/rusty-cv-creator/rusty-cv-config.ini`

```sh
echo DATABASE_URL=sqlite://$HOME/.config/rusty-cv-creator/applications.db > .env
```

## Database setup

Diesel is used to manage the database. To setup the database, run the following command:

```sh
diesel setup
```

## TODO

- [x] Make sure that we source the right config.ini file from `~/.config/rusty-cv-generator/config.ini`
- [x] Make sure that we source the right database file from `~/.config/rusty-cv-generator/database.db`
- Implement `Update`, `Delete`, `Select` and `List`.
  - Implement `List` with a filter.
- Add error matching in important methods.
- Add tests for the all methods.
