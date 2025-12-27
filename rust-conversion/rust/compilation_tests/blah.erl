-module(blah).
-export([factorial/1,facume/1]).


factorial(0) -> 1;
factorial(N) -> N*factorial(N-1).


facume(N) -> facume(N,1).

facume(0,Accum) -> Accum;
facume(N,Accum) -> facume(N-1,Accum * N).
