Card Format (.crd)
------------

A tool for writing playing cards collectible card games.

This format exists to make describing cards for games simpler, though it could be used to describe other distinct items.

It is distinct from JSON in a few ways.

* Comments
* Parameters
* Default Properties
* Predefined Constants

First, because it is intended to be directly edited by humans, it allows for comments

```
#Anything after a '#' on a line is considered a comment
```

Second, it assumes cards will need many of the same properties and makes it easier to add them.

```
@param cost strength

#Archers cost and strength will be 3 and 4 respectively.
Archer 3 4:
.text: "Do something"
```

Third, by setting defaults, you can save writing.

```
@def
.health:3

#Dave will have a health of 3
Dave:
.size:4

#Alan will have health of 4
Alan:
.size:7
.health:4

```

Complicated properties can be predefined for reuse.

```

@const mine_provides : [[wood,4],[metal,3]]

Mine :
.provides:$mine_provides
```



## Format changes in 0.2.0

* "var","param",and "def" were keywords which could have caused mistakes when writing if forgotten. This was awkward and I am much happier having them behind an '@' marker.

* Due to other changes there is currently no way to extend another item by name. It was not a feature I used. You can however redefine the default '@def' at any time, and this will provide a prototype for others to copy.










