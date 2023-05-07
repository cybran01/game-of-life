# Game of Life
A simple implementation of Conways Game of Life in Rust, with a GUI powered by FLTK.

The simulation grid can be dragged via left mouse as well as zoomed via the scroll wheel.
When the simulation is stopped, right mouse can be used to draw directly on the grid or alternatively to place a custom pre-drawn shape that can be selected.

The shapes are loaded from files located in the ```./shapes/``` directory. 
They have a very simple format:
Every character (with the exeption of the newline characters ```\0d\0a```, which just get ignored) corresponds to the value of a cell:
* ```0``` means the cell will be set to being dead.
* ```1``` means the cell will be set to being alive.
* Any other character means that the current value of the cell will not be changed.

For example, consider the following three versions of a glider-shape:
````
 1   010  a1t
  1  001  f31
111  111  111
````
By the above rules, the first and third have identical behavior.
All three versions will produce a glider provided the cells they are placed on are empty. 
If they are not, the first and third one will not produce the shape of a glider;
the second will.
