# How to build
You need to run local PostgreSQL instance to allow compiler to check SQL quieries in advance:
1. Create `hexstody` user with `hexstody` password.
2. Allow database creation for the user. That is required for temporary databases for tests. `ALTER USER hexstody CREATEDB;`
3. Create `hexstody` database and add `hexstody` as owner.

Run build:
```
DATABASE_URL=postgresql://hexstody:hexstody@localhost:5432/hexstody cargo build
```

Run tests:
```
DATABASE_URL=postgresql://hexstody:hexstody@localhost:5432/hexstody cargo test
```

Next, I will assume that `DATABASE_URL` is accessible by cargo commands.

# Swagger

The binary can produce OpenAPI specification:
```
cargo run --bin hexstody-hot -- swagger-public
```