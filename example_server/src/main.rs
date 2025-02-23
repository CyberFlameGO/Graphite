use std::sync::mpsc::Sender;

use command::brigadier;
use command::types::CommandResult;
use concierge::Concierge;
use concierge::ConciergeService;
use net::network_handler::UninitializedConnection;
use protocol::types::GameProfile;
use rand::Rng;
use server::entity::components::BasicEntity;
use server::entity::components::PlayerNPC;
use server::entity::components::Spinalla;
use server::entity::position::Coordinate;
use server::entity::position::Position;
use server::entity::position::Rotation;
use server::inventory::inventory_handler::InventoryHandler;
use server::inventory::inventory_handler::PlayerInventorySection;
use server::inventory::inventory_handler::VanillaPlayerInventory;
use server::player::player_connection::ConnectionReference;
use server::player::player_vec::PlayerVec;
use server::player::proto_player::ProtoPlayer;
use server::player::Player;
use server::player::PlayerService;
use server::universe::Universe;
use server::universe::UniverseService;
use server::world::TickPhase;
use server::world::World;
use server::world::WorldService;

struct MyConciergeImpl {
    universe_sender: Sender<(UninitializedConnection, GameProfile)>,
}

impl ConciergeService for MyConciergeImpl {
    fn get_serverlist_response(&mut self) -> String {
        "{\
            \"version\": {
                \"name\": \"1.19.1\",
                \"protocol\": 760
            },
            \"players\": {
                \"max\": 0,
                \"online\": 0,
                \"sample\": []
            },
            \"description\": {
                \"text\": \"Hello world\"
            },
            \"favicon\": \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAABGklEQVRo3u2aUQ7EIAhEbcNReiPP6Y16F/djk/1bozJASYffJu08BRxMj957yRxnSR4EIMDbAQTylrvWwdOrNTuAY6+NjhV7YiwDbEg3xVgDUKq3wIgp4rtW1FqYAEwuMAQDk0L/FE/q02TUqVR/tTb4vGkDBaTQjL4xIU/i91gJVNeDV8gZ+HnIorAGCJAAwKIBAACAhixyIvsyKL3Qg0bKqzXnbZlNoXmH/NwitvBkeuC1Ira2lk5daBvDAn6/iH9qAi+Fyva9EDDvlYTxVkJZx/RCBMgHgO1L3IEXAmANn+SV7r0DRk5b0im2BfAfaCRcn/JYkBIXwXejDzmPJZ1iVwCHAfrgD08EIAABCEAAAhCAAAQgwG58AEFWdXlZzlUbAAAAAElFTkSuQmCC\"
        }".into()
    }

    fn accept_player(
        &mut self,
        player_connection: UninitializedConnection,
        mut concierge_connection: concierge::ConciergeConnection<Self>,
    ) {
        let join_data = (
            player_connection,
            concierge_connection.game_profile.take().unwrap(),
        );

        self.universe_sender.send(join_data).unwrap();
    }
}

fn main() {
    /*use binary::slice_serialization::BigEndian;
    use binary::slice_serialization::SizedString;
    use binary::slice_serialization::SliceSerializable;

    macros::slice_serializable!(
        #[derive(Debug)]
        pub enum Hello {
            Xyz {
                something: i32 as BigEndian
            },
            Abc {
                something: i32 as BigEndian
            }
        }
    );

    macros::slice_serializable!(
        #[derive(Debug)]
        pub enum Hello<'b> {
            Xyz {
                blah: &'b str as SizedString,
                something: i32 as BigEndian
            },
            Abc {
                blah: &'b str as SizedString,
                something: i32 as BigEndian
            }
        }
    );

    let abc = Hello::Abc {
        blah: "HELLO",
        something: 5
    };

    println!("Write size: {}", Hello::get_write_size(&abc));
    let mut buf = [0; 128];
    unsafe { Hello::write(&mut buf, &abc); }
    println!("Buf: {:?}", buf);
    let out = Hello::read(&mut &buf[..10]);
    println!("out: {:?}", out);*/

    //println!("{:?}", packet);
    //dispatcher.dispatch("hello 800 10");

    // server::command::dispatcher::dispatch("hello 100 whatever_we_want 7174");
    #[brigadier("hello", {10..2000}, {})]
    fn my_function(player: &mut Player<MyPlayerService>, number: u16, numer2: u8) -> CommandResult {
        println!("number: {}", number);
        println!("numer2: {}", numer2);
        player.send_message("Hello from my_function");
        Ok(())
    }

    // Options
    // 1. Take a direct Player<MyPlayerService>
    // 2. Make the function "generic", and use `brigadier_player_types` to specify the service types
    // ---2 generates (in surrogate function)
    /*
    let __player_id_0 = std::any::TypeId::of::<XYZ>();
    let __player_id_1 = std::any::TypeId::of::<ABC>();
    match __player_id {
        __player_id_0 => {
            my_function(player as &mut Player<XYZ>, ...);
        }
        __player_id_1 => {
            my_function(player as &mut Player<ABC>, ...);
        }
    }
    */

    #[brigadier("entity_test", {})]
    fn entity_test(player: &mut Player<MyPlayerService>, entity_type: u8) -> CommandResult {
        player.send_message("Hello from MyPlayerService");

        for _ in 0..1000 {
            let entity_id = player.get_world_mut().get_universe().new_entity_id();

            let test_entity = BasicEntity {
                entity_id,
                entity_type: entity_type as _,
            };

            let entity = (Spinalla {
                direction: (
                    rand::thread_rng().gen_range(-1.0..1.0),
                    rand::thread_rng().gen_range(-1.0..1.0),
                ),
                rotation: Rotation {
                    yaw: 0.0,
                    pitch: 0.0,
                },
            },);

            player.get_world_mut().push_entity(
                entity,
                Coordinate {
                    x: player.position.coord.x,
                    y: player.position.coord.y,
                    z: player.position.coord.z,
                },
                test_entity,
                entity_id,
            );
        }

        Ok(())
    }

    #[brigadier("spawn_player")]
    fn spawn_player(player: &mut Player<MyPlayerService>) -> CommandResult {
        let entity_id = player.get_world_mut().get_universe().new_entity_id();

        let player_npc = PlayerNPC {
            entity_id,
            uuid: rand::thread_rng().gen(),
            username: "Friend".into(),
        };

        let entity = (Spinalla {
            direction: (
                rand::thread_rng().gen_range(-1.0..1.0),
                rand::thread_rng().gen_range(-1.0..1.0),
            ),
            rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
        },);

        player.get_world_mut().push_entity(
            entity,
            Coordinate {
                x: player.position.coord.x,
                y: player.position.coord.y,
                z: player.position.coord.z,
            },
            player_npc,
            entity_id,
        );
        Ok(())
    }

    #[brigadier("gib", {})]
    fn gib(player: &mut Player<MyPlayerService>, slot: u8) -> CommandResult {
        let itemstack = player.inventory.get(PlayerInventorySection::Hotbar(slot as _)).unwrap();
        println!("In slot: {:?}", itemstack);

        Ok(())
    }

    my_function.merge(entity_test).unwrap();
    my_function.merge(spawn_player).unwrap();
    my_function.merge(gib).unwrap();

    let (dispatcher, packet) =
        command::minecraft::create_dispatcher_and_brigadier_packet(my_function);

    let universe_sender = Universe::create_and_start(
        || MyUniverseService {
            the_world: World::new(MyWorldService {
                players: PlayerVec::new(),
            }),
        },
        Some((dispatcher, packet)),
    );

    Concierge::bind("127.0.0.1:25565", MyConciergeImpl { universe_sender }).unwrap();
}

// universe

struct MyUniverseService {
    the_world: World<MyWorldService>,
}

impl UniverseService for MyUniverseService {
    type ConnectionReferenceType = ConnectionReference<Self>;

    fn handle_player_join(universe: &mut Universe<Self>, proto_player: ProtoPlayer<Self>) {
        universe.service.the_world.handle_player_join(proto_player);
    }

    fn initialize(universe: &Universe<Self>) {
        universe.service.the_world.initialize(universe);
    }

    fn tick(universe: &mut Universe<Self>) {
        universe.service.the_world.tick();
    }

    fn get_player_count(universe: &Universe<Self>) -> usize {
        MyWorldService::get_player_count(&universe.service.the_world)
    }
}

// world

struct MyWorldService {
    players: PlayerVec<MyPlayerService>,
}

impl WorldService for MyWorldService {
    type UniverseServiceType = MyUniverseService;
    const CHUNKS_X: usize = 6;
    const CHUNKS_Z: usize = 6;
    const CHUNK_VIEW_DISTANCE: u8 = 8;
    const ENTITY_VIEW_DISTANCE: u8 = 1;

    fn handle_player_join(
        world: &mut World<Self>,
        mut proto_player: ProtoPlayer<Self::UniverseServiceType>,
    ) {
        proto_player.hardcore = true;

        // make player from proto_player
        world
            .service
            .players
            .add(
                proto_player,
                MyPlayerService {},
                Position {
                    coord: Coordinate {
                        x: 32.0,
                        y: 224.0,
                        z: 32.0,
                    },
                    rot: Rotation {
                        yaw: 0.0,
                        pitch: 0.0,
                    },
                },
            )
            .unwrap();
    }

    fn initialize(world: &World<Self>) {
        world.service.players.initialize(world);
    }

    fn tick(world: &mut World<Self>, tick_phase: TickPhase) {
        world.service.players.tick(tick_phase);
    }

    fn get_player_count(world: &World<Self>) -> usize {
        world.service.players.len()
    }
}

// player

struct MyPlayerService {}

impl PlayerService for MyPlayerService {
    type UniverseServiceType = MyUniverseService;
    type WorldServiceType = MyWorldService;
    type InventoryHandlerType = VanillaPlayerInventory;
}
