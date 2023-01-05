# Camden server

## Disclaimer

This server is a successor of the [Simwatch server](https://github.com/vatsimnerd/simwatch), completely rewritten in Rust.

## What does it do?

The purpose of Camden server is to power https://simwatch.vatsimnerd.com which essentially is a VatSpy-like web application. The closest equivalent would be [SimAware](https://map.vatsim.net)

At first Simwatch used to be much more superior than the original [SimAware](https://map.vatsim.net) but they seem to have caught up so nowadays simwatch should be treated as a personal pet-project of mine rather than SimAware competitor. However, it still has certain features SimAware lacks.

## Architecture (kind of)

At start Camden loads vatsim-related static data from [VatSpy Data Project](https://github.com/vatsimnetwork/vatspy-data-project) as well as from other sources like [OurAirports](https://ourairports.com/).

After loading the static data it starts polling vatsim real-time API every `[configurable]` seconds to fetch pilots and controllers presented online. Controllers are then merged with the corresponding static objects like airports and FIRs while pilots' coordinates/altitude/heading are synced to a database (currently being MongoDB) to save flight tracks.

### Get data from Camden

Most data is served via Server-sent events API `/api/updates/<min_lng>/<min_lat>/<max_lng>/<max_lat>` so the frontend can get updates pushed from the server when they're ready. The coordinates in the API path define a map window to track updates within.

### Rest API

`/api/pilots/<callsign>` returns a pilot object with a given callsing if they're online. Unlike the updates API this will also include the pilot's track - a list of track points with the pilot's coordinates and other saved flight data.

`/api/airports/<ICAO or IATA>` searches for an airport. This can be used to fetch uncontrolled airports while the updates API only pushes the controlled ones.

`/api/chkquery?query=...` checks if the pilot's filter is correct. Filtering involves complex things like lexer/parser/compiler/evaluater and every stage may produce errors. This handler is useful for the frontend part so the app is sure the filter is correct before re-requesting updates.

`/api/__build__` contains internal metadata like like package name and version

## What else

Camden server is under development and it still lacks some SimWatch features like the aircraft type database.

**Warning**: you are unlikely to meet a good Rust code example here as I'm in the very beginning of learning Rust. On the other hand, the code might be a bit easier to read than one written by experienced Rust developers.
