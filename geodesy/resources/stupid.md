# Stupid ways of doing things - but useful for testing

## Stupid way of adding three

```geodesy:way_three;
addone | addone inv | addone | addone | addone
```

## Yep! - adding two, too

```geodesy:way_too;
addone | addone inv | addone | addone
```

## Another name for a stupid way of adding two

```geodesy:way_two;

addone | addone inv | addone | addone
```

## Make Helmert do the hard work

```geodesy:addone;
helmert x=1
```

## Add one unless a different value for x is supplied

```geodesy:add_x;
helmert x=*1
```

## Add whichever value of 'something' is supplied

```geodesy:add_something;
helmert x=$something
```

## And use the ones above in stupid ways

```geodesy:addthree_one_by_one;
stupid:addone | stupid:addone | stupid:add_x x=-1 | stupid:add_x x=2
```

```geodesy:addthree;
stupid:addone | stupid:add_something something=2
```

```geodesy:bad;
stupid:addone | stupid:add_something
```

## Tests

```console
$ echo 55 12 | cargo r -- stupid:bad
> Error: Syntax error: 'Incomplete definition for 'x' ('something' not found)'
```

```console
$ echo 55 12 | cargo r -- stupid:addthree
> 58.0000000000 12.0000000000
```

```console
$ echo 55 12 | cargo r -- stupid:addthree_one_by_one
> 58.0000000000 12.0000000000
```
