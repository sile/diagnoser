-module(hello).

-export([world/1]).

-spec world(term()) -> ok.
world(Name) ->
    io:format("Hello World: ~p\n", [Name]),
    ok.
