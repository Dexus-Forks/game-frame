//! The numbers of Direct3D 3's immediate mode, as its public documentation names them:
//! execute-buffer opcodes, render-state types and their values.

/// `D3DOPCODE`: the first byte of a `D3DINSTRUCTION`.
pub mod op {
    pub const POINT: u8 = 1;
    pub const LINE: u8 = 2;
    pub const TRIANGLE: u8 = 3;
    pub const MATRIXLOAD: u8 = 4;
    pub const MATRIXMULTIPLY: u8 = 5;
    pub const STATETRANSFORM: u8 = 6;
    pub const STATELIGHT: u8 = 7;
    pub const STATERENDER: u8 = 8;
    pub const PROCESSVERTICES: u8 = 9;
    pub const TEXTURELOAD: u8 = 10;
    pub const EXIT: u8 = 11;
    pub const BRANCHFORWARD: u8 = 12;
    pub const SPAN: u8 = 13;
    pub const SETSTATUS: u8 = 14;
}

/// `D3DRENDERSTATETYPE`, the states a `D3DOP_STATERENDER` sets.
pub mod rs {
    pub const TEXTUREHANDLE: u32 = 1;
    pub const ANTIALIAS: u32 = 2;
    pub const TEXTUREADDRESS: u32 = 3;
    pub const TEXTUREPERSPECTIVE: u32 = 4;
    pub const WRAPU: u32 = 5;
    pub const WRAPV: u32 = 6;
    pub const ZENABLE: u32 = 7;
    pub const FILLMODE: u32 = 8;
    pub const SHADEMODE: u32 = 9;
    pub const LINEPATTERN: u32 = 10;
    pub const MONOENABLE: u32 = 11;
    pub const ROP2: u32 = 12;
    pub const PLANEMASK: u32 = 13;
    pub const ZWRITEENABLE: u32 = 14;
    pub const ALPHATESTENABLE: u32 = 15;
    pub const LASTPIXEL: u32 = 16;
    pub const TEXTUREMAG: u32 = 17;
    pub const TEXTUREMIN: u32 = 18;
    pub const SRCBLEND: u32 = 19;
    pub const DESTBLEND: u32 = 20;
    pub const TEXTUREMAPBLEND: u32 = 21;
    pub const CULLMODE: u32 = 22;
    pub const ZFUNC: u32 = 23;
    pub const ALPHAREF: u32 = 24;
    pub const ALPHAFUNC: u32 = 25;
    pub const DITHERENABLE: u32 = 26;
    /// `D3DRENDERSTATE_BLENDENABLE`; DirectX 5 calls it `ALPHABLENDENABLE`.
    pub const BLENDENABLE: u32 = 27;
    pub const FOGENABLE: u32 = 28;
    pub const SPECULARENABLE: u32 = 29;
    pub const ZVISIBLE: u32 = 30;
    pub const SUBPIXEL: u32 = 31;
    pub const SUBPIXELX: u32 = 32;
    pub const STIPPLEDALPHA: u32 = 33;
    pub const FOGCOLOR: u32 = 34;
    pub const FOGTABLEMODE: u32 = 35;
    pub const FOGTABLESTART: u32 = 36;
    pub const FOGTABLEEND: u32 = 37;
    pub const FOGTABLEDENSITY: u32 = 38;
    pub const STIPPLEENABLE: u32 = 39;
    /// The 32 rows of the stipple pattern, `STIPPLEPATTERN00` to `STIPPLEPATTERN31`.
    pub const STIPPLEPATTERN00: u32 = 64;
}

/// `D3DSHADEMODE`.
pub const SHADE_FLAT: u32 = 1;
pub const SHADE_GOURAUD: u32 = 2;

/// `D3DFILLMODE`.
pub const FILL_POINT: u32 = 1;
pub const FILL_WIREFRAME: u32 = 2;
pub const FILL_SOLID: u32 = 3;

/// `D3DCULL`.
pub const CULL_NONE: u32 = 1;
pub const CULL_CW: u32 = 2;
pub const CULL_CCW: u32 = 3;

/// `D3DCMPFUNC`, for `ZFUNC` and `ALPHAFUNC`.
pub const CMP_NEVER: u32 = 1;
pub const CMP_LESS: u32 = 2;
pub const CMP_EQUAL: u32 = 3;
pub const CMP_LESSEQUAL: u32 = 4;
pub const CMP_GREATER: u32 = 5;
pub const CMP_NOTEQUAL: u32 = 6;
pub const CMP_GREATEREQUAL: u32 = 7;
pub const CMP_ALWAYS: u32 = 8;

/// `D3DBLEND`, for `SRCBLEND` and `DESTBLEND`.
pub const BLEND_ZERO: u32 = 1;
pub const BLEND_ONE: u32 = 2;
pub const BLEND_SRCCOLOR: u32 = 3;
pub const BLEND_INVSRCCOLOR: u32 = 4;
pub const BLEND_SRCALPHA: u32 = 5;
pub const BLEND_INVSRCALPHA: u32 = 6;
pub const BLEND_DESTALPHA: u32 = 7;
pub const BLEND_INVDESTALPHA: u32 = 8;
pub const BLEND_DESTCOLOR: u32 = 9;
pub const BLEND_INVDESTCOLOR: u32 = 10;
pub const BLEND_SRCALPHASAT: u32 = 11;
pub const BLEND_BOTHSRCALPHA: u32 = 12;
pub const BLEND_BOTHINVSRCALPHA: u32 = 13;

/// `D3DTEXTUREBLEND`, for `TEXTUREMAPBLEND`.
pub const TBLEND_DECAL: u32 = 1;
pub const TBLEND_MODULATE: u32 = 2;
pub const TBLEND_DECALALPHA: u32 = 3;
pub const TBLEND_MODULATEALPHA: u32 = 4;
pub const TBLEND_DECALMASK: u32 = 5;
pub const TBLEND_MODULATEMASK: u32 = 6;
pub const TBLEND_COPY: u32 = 7;
/// DirectX 5.
pub const TBLEND_ADD: u32 = 8;

/// `D3DTEXTUREFILTER`, for `TEXTUREMAG` and `TEXTUREMIN`.
pub const FILTER_NEAREST: u32 = 1;
pub const FILTER_LINEAR: u32 = 2;
pub const FILTER_MIPNEAREST: u32 = 3;
pub const FILTER_MIPLINEAR: u32 = 4;
pub const FILTER_LINEARMIPNEAREST: u32 = 5;
pub const FILTER_LINEARMIPLINEAR: u32 = 6;

/// `D3DTEXTUREADDRESS`.
pub const ADDRESS_WRAP: u32 = 1;
pub const ADDRESS_MIRROR: u32 = 2;
pub const ADDRESS_CLAMP: u32 = 3;

/// `D3DFOGMODE`, for `FOGTABLEMODE`.
pub const FOG_NONE: u32 = 0;

/// `D3DPROCESSVERTICES_*` flags of a `D3DOP_PROCESSVERTICES`.
pub const PROCESS_TRANSFORMLIGHT: u32 = 0;
pub const PROCESS_TRANSFORM: u32 = 1;
pub const PROCESS_COPY: u32 = 2;
pub const PROCESS_OPMASK: u32 = 7;
