-module(hello).

-export([world/0]).

-spec world() -> ok.
world() ->
    io:format("Hello World"),
    ok.
