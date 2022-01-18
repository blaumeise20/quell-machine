use crate::game::cells::CellType;

macro_rules! celld {
    {$(
        $id_name:ident $id:literal {
            $name:literal,
            $description:literal,
            sides $sides:literal,
            texture $texture_name:literal
        }
    )*} => {
        $( pub const $id_name: CellType = $id; )*
        pub static CELL_DATA: &[CellData] = &[
            $(
                CellData {
                    id: $id_name,
                    name: $name,
                    description: $description,
                    sides: $sides,
                    texture_name: $texture_name,
                },
            )*
        ];
    }
}
macro_rules! hotbar {
    ($([$($name:ident),*]),* $(,)?) => {
        pub static HOTBAR_ITEMS: &[&[CellData]] = &[
            $( &[ $(CELL_DATA[($name - 1) as usize], )* ], )*
        ];
    };
}

celld! {
    WALL 1 {
        "Wall",
        "A solid wall that can't be moved by anything.",
        sides 1,
        texture "wall"
    }
    MOVER 2 {
        "Mover",
        "Pushes the cells in front of it.",
        sides 4,
        texture "mover"
    }
    PULLER 3 {
        "Puller",
        "Pulls the cells behind it.",
        sides 4,
        texture "puller"
    }
    PULLSHER 4 {
        "Pullsher",
        "Pulls the cells behind it and pushes the cells in front of it.",
        sides 4,
        texture "pullsher"
    }
    GENERATOR 5 {
        "Generator",
        "Generates the cell behind to its front.",
        sides 4,
        texture "generator"
    }
    ROTATOR_CW 6 {
        "Rotator CW",
        "Rotates all touching cells clockwise.",
        sides 1,
        texture "rotator_cw"
    }
    ROTATOR_CCW 7 {
        "Rotator CCW",
        "Rotates all touching cells counter-clockwise.",
        sides 1,
        texture "rotator_ccw"
    }
    ORIENTATOR 8 {
        "Orientator",
        "Rotates all touching cells in its own direction.",
        sides 4,
        texture "orientator"
    }
    PUSH 9 {
        "Push",
        "A normal cell that does nothing.",
        sides 1,
        texture "push"
    }
    SLIDE 10 {
        "Slide",
        "Like push cell but can only be moved in two directions.",
        sides 2,
        texture "slide"
    }
    TRASH 11 {
        "Trash",
        "Trashes all cells that get moved into it.",
        sides 1,
        texture "trash"
    }
    ENEMY 12 {
        "Enemy",
        "An enemy that moves randomly. *thanks github copilot*",
        sides 1,
        texture "enemy"
    }
    MIRROR 13 {
        "Mirror",
        "Flips the cell in front and behind.",
        sides 2,
        texture "mirror"
    }
    CROSSMIRROR 14 {
        "Cross-Mirror",
        "Like mirror but stacked 90 degrees.",
        sides 1,
        texture "crossmirror"
    }
    TRASHMOVER 15 {
        "Trash Mover",
        "Like a mover but deletes all cells in front of it.",
        sides 4,
        texture "trashmover"
    }
    SPEED 16 {
        "Speed",
        "Fast mover but can't push.",
        sides 4,
        texture "speed"
    }
    MOVLER 17 {
        "Movler",
        "Very weird combination of mover and puller. Doesn't push or pull but increases force when being pushed or pulled.",
        sides 4,
        texture "movler"
    }
    ONE_DIR 18 {
        "One Dir",
        "A cell that can only be moved in one direction.",
        sides 4,
        texture "one_dir"
    }
    SLIDE_WALL 19 {
        "Slide Wall",
        "Like slide but can't be rotated from unmovable sides.",
        sides 2,
        texture "slide_wall"
    }
    GENERATOR_CW 20 {
        "Generator CW",
        "Like generator but rotates to it's right side.",
        sides 4,
        texture "generator_cw"
    }
    GENERATOR_CCW 21 {
        "Generator CCW",
        "Like generator but rotates to it's left side.",
        sides 4,
        texture "generator_ccw"
    }
    TRASHPULLER 22 {
        "Trash Puller",
        "Tries to trash the cell behind it, then moves.",
        sides 4,
        texture "trashpuller"
    }
    GHOST 23 {
        "Ghost",
        "A wall that can't be generated.",
        sides 1,
        texture "ghost"
    }
    STONE 24 {
        "Stone",
        "A cell with gravity. Will try to form hills.",
        sides 4,
        texture "stone"
    }
    REPLICATOR 25 {
        "Replicator",
        "A generator that generates the cell in front of it.",
        sides 4,
        texture "replicator"
    }
    SUCKER 26 {
        "Sucker",
        "Trashes the cell in front of it.",
        sides 4,
        texture "sucker"
    }
    GENERATOR_CROSS 27 {
        "Generator Cross",
        "Like a generator but stacked 90 degrees.",
        sides 4,
        texture "generator_cross"
    }
    MAILBOX 28 {
        "Mailbox",
        "Can be filled with a cell and then moves with it. If it hits a wall it deletes itself and pops out the stored cell.",
        sides 4,
        texture "mailbox"
    }
    POSTOFFICE 29 {
        "Post Office",
        "Used to fill a mailbox. If there is a mailbox in front of it and a movable cell behind it, the \"mail\" will be deleted and put into the mailbox.",
        sides 4,
        texture "postoffice"
    }
    PHYSICAL_GENERATOR 30 {
        "Physical Generator",
        "Generates the cell in front of it. If it hits a wall it pushes itself back.",
        sides 4,
        texture "physical_generator"
    }
    ROTATOR_180 31 {
        "Rotator 180",
        "Rotates all touching cells 180 degrees.",
        sides 1,
        texture "rotator_180"
    }
}

hotbar![
    [WALL, GHOST, STONE],
    [MOVER, PULLER, PULLSHER, MOVLER, TRASHMOVER, TRASHPULLER, SPEED],
    [GENERATOR, GENERATOR_CW, GENERATOR_CCW, GENERATOR_CROSS, REPLICATOR, PHYSICAL_GENERATOR],
    [ROTATOR_CW, ROTATOR_CCW, ROTATOR_180, ORIENTATOR],
    [PUSH, SLIDE, ONE_DIR, SLIDE_WALL],
    [TRASH, ENEMY, SUCKER],
    [MAILBOX, POSTOFFICE],
    [MIRROR, CROSSMIRROR],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    pub id: CellType,
    pub name: &'static str,
    pub description: &'static str,
    pub sides: usize,
    pub texture_name: &'static str,
}
