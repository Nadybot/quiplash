# quiplash

Very simple HTTP server to provide reproducible quiplash prompts.

## Configuration

Set the `PORT` enviroment variable if needed, apart from that it is zero-config.

## API

`GET /prompt`

This will return a random quiplash prompt. Optional query parameters are available:

`seed`: RNG seed to use for reproducible prompts, defaults to OS randomness

`step`: Step for the RNG, defaults to 0

`allow_non_pg`: Allow non-pg prompts to be returned, defaults to false
