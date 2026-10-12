// Generated from pinned RSProt 1.0.0-ALPHA-20260630. See reference/protocol.
use crate::framing::{PacketDef, Size};
pub static CLIENT: &[PacketDef] = &[
    PacketDef {
        opcode: 51,
        size: Size::Fixed(4),
        name: "IF_BUTTON",
    },
    PacketDef {
        opcode: 47,
        size: Size::Fixed(9),
        name: "IF_BUTTONX",
    },
    PacketDef {
        opcode: 40,
        size: Size::Fixed(10),
        name: "IF_SUBOP",
    },
    PacketDef {
        opcode: 48,
        size: Size::Fixed(16),
        name: "IF_BUTTOND",
    },
    PacketDef {
        opcode: 27,
        size: Size::Fixed(16),
        name: "IF_BUTTONT",
    },
    PacketDef {
        opcode: 56,
        size: Size::U16,
        name: "IF_SCRIPT_TRIGGER",
    },
    PacketDef {
        opcode: 102,
        size: Size::Fixed(4),
        name: "OPNPC1_V2",
    },
    PacketDef {
        opcode: 13,
        size: Size::Fixed(4),
        name: "OPNPC2_V2",
    },
    PacketDef {
        opcode: 19,
        size: Size::Fixed(4),
        name: "OPNPC3_V2",
    },
    PacketDef {
        opcode: 45,
        size: Size::Fixed(4),
        name: "OPNPC4_V2",
    },
    PacketDef {
        opcode: 85,
        size: Size::Fixed(4),
        name: "OPNPC5_V2",
    },
    PacketDef {
        opcode: 0,
        size: Size::Fixed(2),
        name: "OPNPC6",
    },
    PacketDef {
        opcode: 86,
        size: Size::Fixed(11),
        name: "OPNPCT",
    },
    PacketDef {
        opcode: 23,
        size: Size::Fixed(11),
        name: "OPNPCU",
    },
    PacketDef {
        opcode: 73,
        size: Size::Fixed(8),
        name: "OPLOC1_V2",
    },
    PacketDef {
        opcode: 103,
        size: Size::Fixed(8),
        name: "OPLOC2_V2",
    },
    PacketDef {
        opcode: 104,
        size: Size::Fixed(8),
        name: "OPLOC3_V2",
    },
    PacketDef {
        opcode: 15,
        size: Size::Fixed(8),
        name: "OPLOC4_V2",
    },
    PacketDef {
        opcode: 116,
        size: Size::Fixed(8),
        name: "OPLOC5_V2",
    },
    PacketDef {
        opcode: 78,
        size: Size::Fixed(2),
        name: "OPLOC6",
    },
    PacketDef {
        opcode: 62,
        size: Size::Fixed(15),
        name: "OPLOCT",
    },
    PacketDef {
        opcode: 52,
        size: Size::Fixed(15),
        name: "OPLOCU",
    },
    PacketDef {
        opcode: 46,
        size: Size::Fixed(8),
        name: "OPOBJ1_V2",
    },
    PacketDef {
        opcode: 74,
        size: Size::Fixed(8),
        name: "OPOBJ2_V2",
    },
    PacketDef {
        opcode: 7,
        size: Size::Fixed(8),
        name: "OPOBJ3_V2",
    },
    PacketDef {
        opcode: 31,
        size: Size::Fixed(8),
        name: "OPOBJ4_V2",
    },
    PacketDef {
        opcode: 43,
        size: Size::Fixed(8),
        name: "OPOBJ5_V2",
    },
    PacketDef {
        opcode: 58,
        size: Size::Fixed(6),
        name: "OPOBJ6",
    },
    PacketDef {
        opcode: 76,
        size: Size::Fixed(15),
        name: "OPOBJT",
    },
    PacketDef {
        opcode: 113,
        size: Size::Fixed(15),
        name: "OPOBJU",
    },
    PacketDef {
        opcode: 93,
        size: Size::Fixed(3),
        name: "OPPLAYER1",
    },
    PacketDef {
        opcode: 3,
        size: Size::Fixed(3),
        name: "OPPLAYER2",
    },
    PacketDef {
        opcode: 92,
        size: Size::Fixed(3),
        name: "OPPLAYER3",
    },
    PacketDef {
        opcode: 37,
        size: Size::Fixed(3),
        name: "OPPLAYER4",
    },
    PacketDef {
        opcode: 57,
        size: Size::Fixed(3),
        name: "OPPLAYER5",
    },
    PacketDef {
        opcode: 70,
        size: Size::Fixed(3),
        name: "OPPLAYER6",
    },
    PacketDef {
        opcode: 22,
        size: Size::Fixed(3),
        name: "OPPLAYER7",
    },
    PacketDef {
        opcode: 53,
        size: Size::Fixed(3),
        name: "OPPLAYER8",
    },
    PacketDef {
        opcode: 79,
        size: Size::Fixed(11),
        name: "OPPLAYERT",
    },
    PacketDef {
        opcode: 87,
        size: Size::Fixed(11),
        name: "OPPLAYERU",
    },
    PacketDef {
        opcode: 14,
        size: Size::Fixed(3),
        name: "OPWORLDENTITY1",
    },
    PacketDef {
        opcode: 80,
        size: Size::Fixed(3),
        name: "OPWORLDENTITY2",
    },
    PacketDef {
        opcode: 100,
        size: Size::Fixed(3),
        name: "OPWORLDENTITY3",
    },
    PacketDef {
        opcode: 83,
        size: Size::Fixed(3),
        name: "OPWORLDENTITY4",
    },
    PacketDef {
        opcode: 106,
        size: Size::Fixed(3),
        name: "OPWORLDENTITY5",
    },
    PacketDef {
        opcode: 98,
        size: Size::Fixed(2),
        name: "OPWORLDENTITY6",
    },
    PacketDef {
        opcode: 72,
        size: Size::Fixed(11),
        name: "OPWORLDENTITYT",
    },
    PacketDef {
        opcode: 90,
        size: Size::Fixed(11),
        name: "OPWORLDENTITYU",
    },
    PacketDef {
        opcode: 29,
        size: Size::Fixed(1),
        name: "EVENT_APPLET_FOCUS",
    },
    PacketDef {
        opcode: 8,
        size: Size::Fixed(4),
        name: "EVENT_CAMERA_POSITION",
    },
    PacketDef {
        opcode: 54,
        size: Size::U16,
        name: "EVENT_KEYBOARD",
    },
    PacketDef {
        opcode: 16,
        size: Size::Fixed(2),
        name: "EVENT_MOUSE_SCROLL",
    },
    PacketDef {
        opcode: 110,
        size: Size::U8,
        name: "EVENT_MOUSE_MOVE",
    },
    PacketDef {
        opcode: 6,
        size: Size::U8,
        name: "EVENT_NATIVE_MOUSE_MOVE",
    },
    PacketDef {
        opcode: 66,
        size: Size::Fixed(6),
        name: "EVENT_MOUSE_CLICK_V1",
    },
    PacketDef {
        opcode: 35,
        size: Size::Fixed(7),
        name: "EVENT_MOUSE_CLICK_V2",
    },
    PacketDef {
        opcode: 115,
        size: Size::Fixed(6),
        name: "RESUME_PAUSEBUTTON",
    },
    PacketDef {
        opcode: 26,
        size: Size::U8,
        name: "RESUME_P_NAMEDIALOG",
    },
    PacketDef {
        opcode: 64,
        size: Size::U8,
        name: "RESUME_P_STRINGDIALOG",
    },
    PacketDef {
        opcode: 75,
        size: Size::Fixed(4),
        name: "RESUME_P_COUNTDIALOG",
    },
    PacketDef {
        opcode: 63,
        size: Size::Fixed(8),
        name: "RESUME_P_COUNTDIALOG_LONG",
    },
    PacketDef {
        opcode: 32,
        size: Size::Fixed(2),
        name: "RESUME_P_OBJDIALOG",
    },
    PacketDef {
        opcode: 101,
        size: Size::U8,
        name: "FRIENDCHAT_KICK",
    },
    PacketDef {
        opcode: 55,
        size: Size::U8,
        name: "FRIENDCHAT_SETRANK",
    },
    PacketDef {
        opcode: 20,
        size: Size::U8,
        name: "FRIENDCHAT_JOIN_LEAVE",
    },
    PacketDef {
        opcode: 112,
        size: Size::Fixed(1),
        name: "CLANCHANNEL_FULL_REQUEST",
    },
    PacketDef {
        opcode: 2,
        size: Size::Fixed(1),
        name: "CLANSETTINGS_FULL_REQUEST",
    },
    PacketDef {
        opcode: 38,
        size: Size::U8,
        name: "CLANCHANNEL_KICKUSER",
    },
    PacketDef {
        opcode: 84,
        size: Size::U8,
        name: "AFFINEDCLANSETTINGS_ADDBANNED_FROMCHANNEL",
    },
    PacketDef {
        opcode: 96,
        size: Size::U8,
        name: "AFFINEDCLANSETTINGS_SETMUTED_FROMCHANNEL",
    },
    PacketDef {
        opcode: 28,
        size: Size::U8,
        name: "FRIENDLIST_ADD",
    },
    PacketDef {
        opcode: 65,
        size: Size::U8,
        name: "FRIENDLIST_DEL",
    },
    PacketDef {
        opcode: 82,
        size: Size::U8,
        name: "IGNORELIST_ADD",
    },
    PacketDef {
        opcode: 9,
        size: Size::U8,
        name: "IGNORELIST_DEL",
    },
    PacketDef {
        opcode: 69,
        size: Size::U8,
        name: "MESSAGE_PUBLIC",
    },
    PacketDef {
        opcode: 111,
        size: Size::U16,
        name: "MESSAGE_PRIVATE",
    },
    PacketDef {
        opcode: 114,
        size: Size::U8,
        name: "MOVE_GAMECLICK",
    },
    PacketDef {
        opcode: 67,
        size: Size::U8,
        name: "MOVE_MINIMAPCLICK",
    },
    PacketDef {
        opcode: 34,
        size: Size::U8,
        name: "CLIENT_CHEAT",
    },
    PacketDef {
        opcode: 59,
        size: Size::Fixed(3),
        name: "SET_CHATFILTERSETTINGS",
    },
    PacketDef {
        opcode: 99,
        size: Size::Fixed(4),
        name: "CLICKWORLDMAP",
    },
    PacketDef {
        opcode: 5,
        size: Size::Fixed(0),
        name: "EXIT_FREECAM",
    },
    PacketDef {
        opcode: 95,
        size: Size::Fixed(0),
        name: "CLOSE_MODAL",
    },
    PacketDef {
        opcode: 50,
        size: Size::Fixed(9),
        name: "TELEPORT",
    },
    PacketDef {
        opcode: 108,
        size: Size::U16,
        name: "BUG_REPORT",
    },
    PacketDef {
        opcode: 60,
        size: Size::U8,
        name: "SEND_SNAPSHOT",
    },
    PacketDef {
        opcode: 17,
        size: Size::U8,
        name: "HISCORE_REQUEST",
    },
    PacketDef {
        opcode: 91,
        size: Size::Fixed(22),
        name: "IF_CRMVIEW_OP",
    },
    PacketDef {
        opcode: 77,
        size: Size::Fixed(26),
        name: "UPDATE_PLAYER_MODEL_V2",
    },
    PacketDef {
        opcode: 97,
        size: Size::U8,
        name: "CONNECTION_TELEMETRY",
    },
    PacketDef {
        opcode: 1,
        size: Size::Fixed(10),
        name: "SEND_PING_REPLY",
    },
    PacketDef {
        opcode: 33,
        size: Size::Fixed(4),
        name: "DETECT_MODIFIED_CLIENT",
    },
    PacketDef {
        opcode: 12,
        size: Size::U8,
        name: "REFLECTION_CHECK_REPLY",
    },
    PacketDef {
        opcode: 89,
        size: Size::Fixed(0),
        name: "NO_TIMEOUT",
    },
    PacketDef {
        opcode: 4,
        size: Size::Fixed(0),
        name: "IDLE",
    },
    PacketDef {
        opcode: 24,
        size: Size::Fixed(0),
        name: "MAP_BUILD_COMPLETE",
    },
    PacketDef {
        opcode: 61,
        size: Size::Fixed(2),
        name: "MEMBERSHIP_PROMOTION_ELIGIBILITY",
    },
    PacketDef {
        opcode: 39,
        size: Size::Fixed(4),
        name: "SOUND_JINGLEEND",
    },
    PacketDef {
        opcode: 10,
        size: Size::Fixed(5),
        name: "WINDOW_STATUS",
    },
    PacketDef {
        opcode: 44,
        size: Size::Fixed(1),
        name: "SET_HEADING",
    },
    PacketDef {
        opcode: 105,
        size: Size::Fixed(1),
        name: "RSEVEN_STATUS",
    },
];
// Reference entries with no standalone wire opcode; not silently mapped to 255.
pub static CLIENT_UNASSIGNED: &[&str] = &[];
pub static SERVER: &[PacketDef] = &[
    PacketDef {
        opcode: 25,
        size: Size::U16,
        name: "IF_RESYNC_V2",
    },
    PacketDef {
        opcode: 96,
        size: Size::Fixed(2),
        name: "IF_OPENTOP",
    },
    PacketDef {
        opcode: 7,
        size: Size::Fixed(7),
        name: "IF_OPENSUB",
    },
    PacketDef {
        opcode: 23,
        size: Size::Fixed(4),
        name: "IF_CLOSESUB",
    },
    PacketDef {
        opcode: 123,
        size: Size::Fixed(8),
        name: "IF_MOVESUB",
    },
    PacketDef {
        opcode: 94,
        size: Size::Fixed(4),
        name: "IF_CLEARINV",
    },
    PacketDef {
        opcode: 108,
        size: Size::Fixed(16),
        name: "IF_SETEVENTS_V2",
    },
    PacketDef {
        opcode: 102,
        size: Size::Fixed(8),
        name: "IF_SETPOSITION",
    },
    PacketDef {
        opcode: 82,
        size: Size::Fixed(6),
        name: "IF_SETSCROLLPOS",
    },
    PacketDef {
        opcode: 14,
        size: Size::Fixed(8),
        name: "IF_SETROTATESPEED",
    },
    PacketDef {
        opcode: 80,
        size: Size::U16,
        name: "IF_SETTEXT",
    },
    PacketDef {
        opcode: 63,
        size: Size::Fixed(5),
        name: "IF_SETHIDE",
    },
    PacketDef {
        opcode: 17,
        size: Size::Fixed(10),
        name: "IF_SETANGLE",
    },
    PacketDef {
        opcode: 106,
        size: Size::Fixed(10),
        name: "IF_SETOBJECT",
    },
    PacketDef {
        opcode: 55,
        size: Size::Fixed(6),
        name: "IF_SETCOLOUR",
    },
    PacketDef {
        opcode: 59,
        size: Size::Fixed(6),
        name: "IF_SETANIM",
    },
    PacketDef {
        opcode: 84,
        size: Size::Fixed(6),
        name: "IF_SETNPCHEAD",
    },
    PacketDef {
        opcode: 89,
        size: Size::Fixed(6),
        name: "IF_SETNPCHEAD_ACTIVE",
    },
    PacketDef {
        opcode: 120,
        size: Size::Fixed(4),
        name: "IF_SETPLAYERHEAD",
    },
    PacketDef {
        opcode: 142,
        size: Size::Fixed(8),
        name: "IF_SETMODEL_V2",
    },
    PacketDef {
        opcode: 16,
        size: Size::Fixed(6),
        name: "IF_SETPLAYERMODEL_BASECOLOUR",
    },
    PacketDef {
        opcode: 100,
        size: Size::Fixed(5),
        name: "IF_SETPLAYERMODEL_BODYTYPE",
    },
    PacketDef {
        opcode: 98,
        size: Size::Fixed(8),
        name: "IF_SETPLAYERMODEL_OBJ",
    },
    PacketDef {
        opcode: 121,
        size: Size::Fixed(5),
        name: "IF_SETPLAYERMODEL_SELF",
    },
    PacketDef {
        opcode: 35,
        size: Size::Fixed(10),
        name: "MIDI_SONG_V2",
    },
    PacketDef {
        opcode: 99,
        size: Size::Fixed(12),
        name: "MIDI_SONG_WITHSECONDARY",
    },
    PacketDef {
        opcode: 107,
        size: Size::Fixed(8),
        name: "MIDI_SWAP",
    },
    PacketDef {
        opcode: 0,
        size: Size::Fixed(4),
        name: "MIDI_SONG_STOP",
    },
    PacketDef {
        opcode: 118,
        size: Size::Fixed(5),
        name: "MIDI_JINGLE",
    },
    PacketDef {
        opcode: 77,
        size: Size::Fixed(5),
        name: "SYNTH_SOUND",
    },
    PacketDef {
        opcode: 145,
        size: Size::Fixed(3),
        name: "AMBIENCE_START",
    },
    PacketDef {
        opcode: 138,
        size: Size::Fixed(1),
        name: "AMBIENCE_STOP",
    },
    PacketDef {
        opcode: 54,
        size: Size::Fixed(3),
        name: "UPDATE_ZONE_FULL_FOLLOWS",
    },
    PacketDef {
        opcode: 9,
        size: Size::Fixed(3),
        name: "UPDATE_ZONE_PARTIAL_FOLLOWS",
    },
    PacketDef {
        opcode: 105,
        size: Size::U16,
        name: "UPDATE_ZONE_PARTIAL_ENCLOSED",
    },
    PacketDef {
        opcode: 33,
        size: Size::U16,
        name: "LOC_ADD_CHANGE_V2",
    },
    PacketDef {
        opcode: 10,
        size: Size::Fixed(2),
        name: "LOC_DEL",
    },
    PacketDef {
        opcode: 110,
        size: Size::Fixed(4),
        name: "LOC_ANIM",
    },
    PacketDef {
        opcode: 34,
        size: Size::Fixed(14),
        name: "LOC_MERGE",
    },
    PacketDef {
        opcode: 86,
        size: Size::Fixed(6),
        name: "MAP_ANIM",
    },
    PacketDef {
        opcode: 32,
        size: Size::Fixed(7),
        name: "SOUND_AREA",
    },
    PacketDef {
        opcode: 48,
        size: Size::Fixed(27),
        name: "PROJANIM_SPECIFIC_V4",
    },
    PacketDef {
        opcode: 90,
        size: Size::Fixed(8),
        name: "MAP_ANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 30,
        size: Size::Fixed(6),
        name: "LOC_ANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 4,
        size: Size::Fixed(9),
        name: "NPC_HEADICON_SPECIFIC",
    },
    PacketDef {
        opcode: 101,
        size: Size::Fixed(9),
        name: "NPC_SPOTANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 20,
        size: Size::Fixed(5),
        name: "NPC_ANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 119,
        size: Size::Fixed(3),
        name: "ANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 72,
        size: Size::Fixed(9),
        name: "PLAYER_SPOTANIM_SPECIFIC",
    },
    PacketDef {
        opcode: 78,
        size: Size::Fixed(17),
        name: "OBJ_ADD_SPECIFIC",
    },
    PacketDef {
        opcode: 41,
        size: Size::Fixed(10),
        name: "OBJ_DEL_SPECIFIC",
    },
    PacketDef {
        opcode: 70,
        size: Size::Fixed(7),
        name: "OBJ_ENABLED_OPS_SPECIFIC",
    },
    PacketDef {
        opcode: 104,
        size: Size::Fixed(10),
        name: "OBJ_UNCUSTOMISE_SPECIFIC",
    },
    PacketDef {
        opcode: 13,
        size: Size::Fixed(14),
        name: "OBJ_COUNT_SPECIFIC",
    },
    PacketDef {
        opcode: 52,
        size: Size::Fixed(20),
        name: "OBJ_CUSTOMISE_SPECIFIC",
    },
    PacketDef {
        opcode: 28,
        size: Size::U16,
        name: "PLAYER_INFO",
    },
    PacketDef {
        opcode: 85,
        size: Size::U16,
        name: "NPC_INFO_SMALL_V5",
    },
    PacketDef {
        opcode: 2,
        size: Size::U16,
        name: "NPC_INFO_LARGE_V5",
    },
    PacketDef {
        opcode: 116,
        size: Size::Fixed(2),
        name: "SET_NPC_UPDATE_ORIGIN",
    },
    PacketDef {
        opcode: 47,
        size: Size::Fixed(3),
        name: "SET_ACTIVE_WORLD_V2",
    },
    PacketDef {
        opcode: 122,
        size: Size::U16,
        name: "WORLDENTITY_INFO_V7",
    },
    PacketDef {
        opcode: 49,
        size: Size::U16,
        name: "REBUILD_NORMAL_V2",
    },
    PacketDef {
        opcode: 125,
        size: Size::U16,
        name: "REBUILD_REGION_V2",
    },
    PacketDef {
        opcode: 109,
        size: Size::U16,
        name: "REBUILD_WORLDENTITY_V4",
    },
    PacketDef {
        opcode: 97,
        size: Size::Fixed(3),
        name: "VARP_SMALL",
    },
    PacketDef {
        opcode: 12,
        size: Size::Fixed(6),
        name: "VARP_LARGE",
    },
    PacketDef {
        opcode: 44,
        size: Size::Fixed(0),
        name: "VARP_RESET",
    },
    PacketDef {
        opcode: 76,
        size: Size::Fixed(0),
        name: "VARP_SYNC",
    },
    PacketDef {
        opcode: 103,
        size: Size::Fixed(4),
        name: "CAM_SHAKE",
    },
    PacketDef {
        opcode: 3,
        size: Size::Fixed(0),
        name: "CAM_RESET",
    },
    PacketDef {
        opcode: 29,
        size: Size::Fixed(4),
        name: "CAM_SMOOTHRESET",
    },
    PacketDef {
        opcode: 146,
        size: Size::Fixed(8),
        name: "CAM_MOVETO_V2",
    },
    PacketDef {
        opcode: 147,
        size: Size::Fixed(9),
        name: "CAM_MOVETO_V3",
    },
    PacketDef {
        opcode: 134,
        size: Size::Fixed(10),
        name: "CAM_MOVETO_CYCLES_V2",
    },
    PacketDef {
        opcode: 130,
        size: Size::Fixed(11),
        name: "CAM_MOVETO_CYCLES_V3",
    },
    PacketDef {
        opcode: 129,
        size: Size::Fixed(14),
        name: "CAM_MOVETO_ARC_V2",
    },
    PacketDef {
        opcode: 128,
        size: Size::Fixed(15),
        name: "CAM_MOVETO_ARC_V3",
    },
    PacketDef {
        opcode: 139,
        size: Size::Fixed(8),
        name: "CAM_LOOKAT_V2",
    },
    PacketDef {
        opcode: 144,
        size: Size::Fixed(9),
        name: "CAM_LOOKAT_V3",
    },
    PacketDef {
        opcode: 137,
        size: Size::Fixed(10),
        name: "CAM_LOOKAT_CYCLES",
    },
    PacketDef {
        opcode: 141,
        size: Size::Fixed(9),
        name: "CAM_ROTATETO_COORDINATE_V2",
    },
    PacketDef {
        opcode: 148,
        size: Size::Fixed(11),
        name: "CAM_ROTATETO_COORDINATE_V3",
    },
    PacketDef {
        opcode: 51,
        size: Size::Fixed(7),
        name: "CAM_ROTATEBY",
    },
    PacketDef {
        opcode: 40,
        size: Size::Fixed(7),
        name: "CAM_ROTATETO",
    },
    PacketDef {
        opcode: 38,
        size: Size::Fixed(1),
        name: "CAM_MODE",
    },
    PacketDef {
        opcode: 1,
        size: Size::Fixed(5),
        name: "CAM_TARGET_V3",
    },
    PacketDef {
        opcode: 87,
        size: Size::Fixed(5),
        name: "CAM_TARGET_V4",
    },
    PacketDef {
        opcode: 65,
        size: Size::Fixed(4),
        name: "OCULUS_SYNC",
    },
    PacketDef {
        opcode: 131,
        size: Size::Fixed(1),
        name: "CAM_UNLOCK",
    },
    PacketDef {
        opcode: 135,
        size: Size::Fixed(4),
        name: "CAM_SKYBOX",
    },
    PacketDef {
        opcode: 132,
        size: Size::U16,
        name: "GROUP_FULL",
    },
    PacketDef {
        opcode: 136,
        size: Size::U16,
        name: "GROUP_VAR",
    },
    PacketDef {
        opcode: 140,
        size: Size::Fixed(9),
        name: "GROUP_VAR_INT",
    },
    PacketDef {
        opcode: 133,
        size: Size::Fixed(13),
        name: "GROUP_VAR_LONG",
    },
    PacketDef {
        opcode: 22,
        size: Size::U16,
        name: "UPDATE_INV_FULL",
    },
    PacketDef {
        opcode: 18,
        size: Size::U16,
        name: "UPDATE_INV_PARTIAL",
    },
    PacketDef {
        opcode: 45,
        size: Size::Fixed(2),
        name: "UPDATE_INV_STOPTRANSMIT",
    },
    PacketDef {
        opcode: 19,
        size: Size::U16,
        name: "MESSAGE_PRIVATE",
    },
    PacketDef {
        opcode: 15,
        size: Size::U16,
        name: "MESSAGE_PRIVATE_ECHO",
    },
    PacketDef {
        opcode: 36,
        size: Size::Fixed(0),
        name: "FRIENDLIST_LOADED",
    },
    PacketDef {
        opcode: 111,
        size: Size::U16,
        name: "UPDATE_FRIENDLIST",
    },
    PacketDef {
        opcode: 42,
        size: Size::U16,
        name: "UPDATE_IGNORELIST",
    },
    PacketDef {
        opcode: 27,
        size: Size::U16,
        name: "UPDATE_FRIENDCHAT_CHANNEL_FULL_V2",
    },
    PacketDef {
        opcode: 112,
        size: Size::U8,
        name: "UPDATE_FRIENDCHAT_CHANNEL_SINGLEUSER",
    },
    PacketDef {
        opcode: 61,
        size: Size::U8,
        name: "MESSAGE_FRIENDCHANNEL",
    },
    PacketDef {
        opcode: 11,
        size: Size::U8,
        name: "VARCLAN",
    },
    PacketDef {
        opcode: 88,
        size: Size::Fixed(0),
        name: "VARCLAN_ENABLE",
    },
    PacketDef {
        opcode: 60,
        size: Size::Fixed(0),
        name: "VARCLAN_DISABLE",
    },
    PacketDef {
        opcode: 69,
        size: Size::U16,
        name: "CLANCHANNEL_FULL",
    },
    PacketDef {
        opcode: 95,
        size: Size::U16,
        name: "CLANCHANNEL_DELTA",
    },
    PacketDef {
        opcode: 81,
        size: Size::U16,
        name: "CLANSETTINGS_FULL",
    },
    PacketDef {
        opcode: 71,
        size: Size::U16,
        name: "CLANSETTINGS_DELTA",
    },
    PacketDef {
        opcode: 127,
        size: Size::U8,
        name: "MESSAGE_CLANCHANNEL",
    },
    PacketDef {
        opcode: 62,
        size: Size::U8,
        name: "MESSAGE_CLANCHANNEL_SYSTEM",
    },
    PacketDef {
        opcode: 57,
        size: Size::Fixed(0),
        name: "LOGOUT",
    },
    PacketDef {
        opcode: 58,
        size: Size::Fixed(1),
        name: "LOGOUT_WITHREASON",
    },
    PacketDef {
        opcode: 91,
        size: Size::U8,
        name: "LOGOUT_TRANSFER",
    },
    PacketDef {
        opcode: 31,
        size: Size::Fixed(2),
        name: "UPDATE_RUNWEIGHT",
    },
    PacketDef {
        opcode: 64,
        size: Size::Fixed(2),
        name: "UPDATE_RUNENERGY",
    },
    PacketDef {
        opcode: 143,
        size: Size::Fixed(4),
        name: "SET_MAP_FLAG_V2",
    },
    PacketDef {
        opcode: 6,
        size: Size::U8,
        name: "SET_PLAYER_OP",
    },
    PacketDef {
        opcode: 46,
        size: Size::Fixed(7),
        name: "UPDATE_STAT_V2",
    },
    PacketDef {
        opcode: 114,
        size: Size::U16,
        name: "RUNCLIENTSCRIPT",
    },
    PacketDef {
        opcode: 117,
        size: Size::Fixed(0),
        name: "TRIGGER_ONDIALOGABORT",
    },
    PacketDef {
        opcode: 74,
        size: Size::U8,
        name: "MESSAGE_GAME",
    },
    PacketDef {
        opcode: 124,
        size: Size::Fixed(2),
        name: "CHAT_FILTER_SETTINGS",
    },
    PacketDef {
        opcode: 5,
        size: Size::Fixed(1),
        name: "CHAT_FILTER_SETTINGS_PRIVATECHAT",
    },
    PacketDef {
        opcode: 56,
        size: Size::U16,
        name: "UPDATE_TRADINGPOST",
    },
    PacketDef {
        opcode: 53,
        size: Size::Fixed(20),
        name: "UPDATE_STOCKMARKET_SLOT",
    },
    PacketDef {
        opcode: 68,
        size: Size::Fixed(8),
        name: "ACCOUNT_FLAGS",
    },
    PacketDef {
        opcode: 50,
        size: Size::Fixed(6),
        name: "HINT_ARROW",
    },
    PacketDef {
        opcode: 92,
        size: Size::Fixed(0),
        name: "RESET_ANIMS",
    },
    PacketDef {
        opcode: 37,
        size: Size::U8,
        name: "UPDATE_REBOOT_TIMER_V2",
    },
    PacketDef {
        opcode: 26,
        size: Size::Fixed(1),
        name: "SET_HEATMAP_ENABLED",
    },
    PacketDef {
        opcode: 43,
        size: Size::Fixed(1),
        name: "MINIMAP_TOGGLE",
    },
    PacketDef {
        opcode: 83,
        size: Size::Fixed(0),
        name: "SERVER_TICK_END",
    },
    PacketDef {
        opcode: 75,
        size: Size::Fixed(1),
        name: "HIDENPCOPS",
    },
    PacketDef {
        opcode: 73,
        size: Size::Fixed(1),
        name: "HIDEOBJOPS",
    },
    PacketDef {
        opcode: 21,
        size: Size::Fixed(1),
        name: "HIDELOCOPS",
    },
    PacketDef {
        opcode: 79,
        size: Size::Fixed(4),
        name: "SET_INTERACTION_MODE",
    },
    PacketDef {
        opcode: 66,
        size: Size::Fixed(2),
        name: "RESET_INTERACTION_MODE",
    },
    PacketDef {
        opcode: 39,
        size: Size::U16,
        name: "URL_OPEN",
    },
    PacketDef {
        opcode: 67,
        size: Size::U8,
        name: "SITE_SETTINGS",
    },
    PacketDef {
        opcode: 113,
        size: Size::Fixed(28),
        name: "UPDATE_UID192",
    },
    PacketDef {
        opcode: 8,
        size: Size::U16,
        name: "REFLECTION_CHECKER",
    },
    PacketDef {
        opcode: 115,
        size: Size::Fixed(8),
        name: "SEND_PING",
    },
    PacketDef {
        opcode: 126,
        size: Size::U16,
        name: "HISCORE_REPLY",
    },
    PacketDef {
        opcode: 93,
        size: Size::Fixed(2),
        name: "PACKET_GROUP_START",
    },
    PacketDef {
        opcode: 24,
        size: Size::U8,
        name: "UNKNOWN_STRING",
    },
];
// Reference entries with no standalone wire opcode; not silently mapped to 255.
pub static SERVER_UNASSIGNED: &[&str] = &[
    "OBJ_ADD",
    "OBJ_DEL",
    "OBJ_COUNT",
    "OBJ_ENABLED_OPS",
    "OBJ_CUSTOMISE",
    "OBJ_UNCUSTOMISE",
    "MAP_PROJANIM_V2",
    "SCRIPTEDPROJ_ADD",
    "SCRIPTEDPROJ_CHANGE",
];
pub static LOGIN_CLIENT: &[PacketDef] = &[
    PacketDef {
        opcode: 14,
        size: Size::Fixed(0),
        name: "INIT_GAME_CONNECTION",
    },
    PacketDef {
        opcode: 15,
        size: Size::Fixed(20),
        name: "INIT_JS5REMOTE_CONNECTION",
    },
    PacketDef {
        opcode: 16,
        size: Size::U16,
        name: "GAMELOGIN",
    },
    PacketDef {
        opcode: 18,
        size: Size::U16,
        name: "GAMERECONNECT",
    },
    PacketDef {
        opcode: 19,
        size: Size::U16,
        name: "POW_REPLY",
    },
    PacketDef {
        opcode: 21,
        size: Size::Fixed(37),
        name: "UNKNOWN",
    },
    PacketDef {
        opcode: 27,
        size: Size::Fixed(0),
        name: "SSL_WEB_CONNECTION",
    },
    PacketDef {
        opcode: 32,
        size: Size::Fixed(66),
        name: "REMAINING_BETA_ARCHIVE_CRCS",
    },
];
// Reference entries with no standalone wire opcode; not silently mapped to 255.
pub static LOGIN_CLIENT_UNASSIGNED: &[&str] = &[];
pub static LOGIN_SERVER: &[PacketDef] = &[
    PacketDef {
        opcode: 0,
        size: Size::Fixed(0),
        name: "SUCCESSFUL",
    },
    PacketDef {
        opcode: 2,
        size: Size::U8,
        name: "OK",
    },
    PacketDef {
        opcode: 3,
        size: Size::Fixed(0),
        name: "INVALID_USERNAME_OR_PASSWORD",
    },
    PacketDef {
        opcode: 4,
        size: Size::Fixed(0),
        name: "BANNED",
    },
    PacketDef {
        opcode: 5,
        size: Size::Fixed(0),
        name: "DUPLICATE",
    },
    PacketDef {
        opcode: 6,
        size: Size::Fixed(0),
        name: "CLIENT_OUT_OF_DATE",
    },
    PacketDef {
        opcode: 7,
        size: Size::Fixed(0),
        name: "SERVER_FULL",
    },
    PacketDef {
        opcode: 8,
        size: Size::Fixed(0),
        name: "LOGINSERVER_OFFLINE",
    },
    PacketDef {
        opcode: 9,
        size: Size::Fixed(0),
        name: "IP_LIMIT",
    },
    PacketDef {
        opcode: 10,
        size: Size::Fixed(0),
        name: "BAD_SESSION_ID",
    },
    PacketDef {
        opcode: 11,
        size: Size::Fixed(0),
        name: "FORCE_PASSWORD_CHANGE",
    },
    PacketDef {
        opcode: 12,
        size: Size::Fixed(0),
        name: "NEED_MEMBERS_ACCOUNT",
    },
    PacketDef {
        opcode: 13,
        size: Size::Fixed(0),
        name: "INVALID_SAVE",
    },
    PacketDef {
        opcode: 14,
        size: Size::Fixed(0),
        name: "UPDATE_IN_PROGRESS",
    },
    PacketDef {
        opcode: 15,
        size: Size::U16,
        name: "RECONNECT_OK",
    },
    PacketDef {
        opcode: 16,
        size: Size::Fixed(0),
        name: "TOO_MANY_ATTEMPTS",
    },
    PacketDef {
        opcode: 17,
        size: Size::Fixed(0),
        name: "IN_MEMBERS_AREA",
    },
    PacketDef {
        opcode: 18,
        size: Size::Fixed(0),
        name: "LOCKED",
    },
    PacketDef {
        opcode: 19,
        size: Size::Fixed(0),
        name: "CLOSED_BETA_INVITED_ONLY",
    },
    PacketDef {
        opcode: 20,
        size: Size::Fixed(0),
        name: "INVALID_LOGINSERVER",
    },
    PacketDef {
        opcode: 21,
        size: Size::Fixed(0),
        name: "HOP_BLOCKED",
    },
    PacketDef {
        opcode: 22,
        size: Size::Fixed(0),
        name: "INVALID_LOGIN_PACKET",
    },
    PacketDef {
        opcode: 23,
        size: Size::Fixed(0),
        name: "LOGINSERVER_NO_REPLY",
    },
    PacketDef {
        opcode: 24,
        size: Size::Fixed(0),
        name: "LOGINSERVER_LOAD_ERROR",
    },
    PacketDef {
        opcode: 25,
        size: Size::Fixed(0),
        name: "UNKNOWN_REPLY_FROM_LOGINSERVER",
    },
    PacketDef {
        opcode: 26,
        size: Size::Fixed(0),
        name: "IP_BLOCKED",
    },
    PacketDef {
        opcode: 27,
        size: Size::Fixed(0),
        name: "SERVICE_UNAVAILABLE",
    },
    PacketDef {
        opcode: 29,
        size: Size::U16,
        name: "DISALLOWED_BY_SCRIPT",
    },
    PacketDef {
        opcode: 31,
        size: Size::Fixed(0),
        name: "DISPLAYNAME_REQUIRED",
    },
    PacketDef {
        opcode: 32,
        size: Size::Fixed(0),
        name: "NEGATIVE_CREDIT",
    },
    PacketDef {
        opcode: 35,
        size: Size::Fixed(0),
        name: "INVALID_SINGLE_SIGNON",
    },
    PacketDef {
        opcode: 36,
        size: Size::Fixed(0),
        name: "NO_REPLY_FROM_SINGLE_SIGNON",
    },
    PacketDef {
        opcode: 37,
        size: Size::Fixed(0),
        name: "PROFILE_BEING_EDITED",
    },
    PacketDef {
        opcode: 38,
        size: Size::Fixed(0),
        name: "NO_BETA_ACCESS",
    },
    PacketDef {
        opcode: 39,
        size: Size::Fixed(0),
        name: "INSTANCE_INVALID",
    },
    PacketDef {
        opcode: 40,
        size: Size::Fixed(0),
        name: "INSTANCE_NOT_SPECIFIED",
    },
    PacketDef {
        opcode: 41,
        size: Size::Fixed(0),
        name: "INSTANCE_FULL",
    },
    PacketDef {
        opcode: 42,
        size: Size::Fixed(0),
        name: "IN_QUEUE",
    },
    PacketDef {
        opcode: 43,
        size: Size::Fixed(0),
        name: "ALREADY_IN_QUEUE",
    },
    PacketDef {
        opcode: 44,
        size: Size::Fixed(0),
        name: "BILLING_TIMEOUT",
    },
    PacketDef {
        opcode: 45,
        size: Size::Fixed(0),
        name: "NOT_AGREED_TO_NDA",
    },
    PacketDef {
        opcode: 47,
        size: Size::Fixed(0),
        name: "EMAIL_NOT_VALIDATED",
    },
    PacketDef {
        opcode: 50,
        size: Size::Fixed(0),
        name: "CONNECT_FAIL",
    },
    PacketDef {
        opcode: 55,
        size: Size::Fixed(0),
        name: "PRIVACY_POLICY",
    },
    PacketDef {
        opcode: 56,
        size: Size::Fixed(0),
        name: "AUTHENTICATOR",
    },
    PacketDef {
        opcode: 57,
        size: Size::Fixed(0),
        name: "INVALID_AUTHENTICATOR_CODE",
    },
    PacketDef {
        opcode: 61,
        size: Size::Fixed(0),
        name: "UPDATE_DOB",
    },
    PacketDef {
        opcode: 62,
        size: Size::Fixed(0),
        name: "TIMEOUT",
    },
    PacketDef {
        opcode: 63,
        size: Size::Fixed(0),
        name: "KICK",
    },
    PacketDef {
        opcode: 64,
        size: Size::Fixed(0),
        name: "RETRY",
    },
    PacketDef {
        opcode: 65,
        size: Size::Fixed(0),
        name: "LOGIN_FAIL_1",
    },
    PacketDef {
        opcode: 67,
        size: Size::Fixed(0),
        name: "LOGIN_FAIL_2",
    },
    PacketDef {
        opcode: 68,
        size: Size::Fixed(0),
        name: "OUT_OF_DATE_RELOAD",
    },
    PacketDef {
        opcode: 69,
        size: Size::U16,
        name: "PROOF_OF_WORK",
    },
    PacketDef {
        opcode: 71,
        size: Size::Fixed(0),
        name: "DOB_ERROR",
    },
    PacketDef {
        opcode: 72,
        size: Size::Fixed(0),
        name: "WEBSITE_DOB",
    },
    PacketDef {
        opcode: 73,
        size: Size::Fixed(0),
        name: "DOB_REVIEW",
    },
    PacketDef {
        opcode: 74,
        size: Size::Fixed(0),
        name: "CLOSED_BETA",
    },
];
// Reference entries with no standalone wire opcode; not silently mapped to 255.
pub static LOGIN_SERVER_UNASSIGNED: &[&str] = &[];
