use crate::game::cells::CellType;

macro_rules! celld {
    {$(
        $id_name:ident $id:literal: {
            $name:literal,
            $description:literal,
            sides $sides:literal,
            texture $texture_name:literal
        }
    )*} => {
        $( pub const $id_name: CellType = $id; )*
        pub static HOTBAR_CELLS: &[CellType] = &[
            $($id_name, )*
        ];
    }
}

celld! {
    WALL 1: {
        "Wall",
        "A solid wall that can't be moved by anything.",
        sides 1,
        texture "wall"
    }
    MOVER 2: {
        "Mover",
        "Pushes the cells in front of it.",
        sides 4,
        texture "mover"
    }
    PULLER 3: {
        "Puller",
        "Pulls the cells behind it.",
        sides 4,
        texture "puller"
    }
    PULLSHER 4: {
        "Pullsher",
        "Pulls the cells behind it and pushes the cells in front of it.",
        sides 4,
        texture "pullsher"
    }
    GENERATOR 5: {
        "Generator",
        "Generates the cell behind to its front.",
        sides 4,
        texture "generator"
    }
    ROTATOR_CW 6: {
        "Rotator CW",
        "Rotates all touching cells clockwise.",
        sides 1,
        texture "rotator_cw"
    }
    ROTATOR_CCW 7: {
        "Rotator CCW",
        "Rotates all touching cells counter-clockwise.",
        sides 1,
        texture "rotator_ccw"
    }
    ORIENTATOR 8: {
        "Orientator",
        "Rotates all touching cells in its own direction.",
        sides 4,
        texture "orientator"
    }
    PUSH 9: {
        "Push",
        "A normal cell that does nothing.",
        sides 1,
        texture "push"
    }
    SLIDE 10: {
        "Slide",
        "Like push cell but can only be moved in two directions.",
        sides 2,
        texture "slide"
    }
    TRASH 11: {
        "Trash",
        "Trashes all cells that get moved into it.",
        sides 1,
        texture "trash"
    }
    ENEMY 12: {
        "Enemy",
        "An enemy that moves randomly. *thanks github copilot*",
        sides 1,
        texture "enemy"
    }
}
