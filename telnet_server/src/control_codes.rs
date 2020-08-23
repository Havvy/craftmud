#[repr(u8)]
enum ControlCode {
    /// End of subnegotiation parameters.
    SE = 240,

    /// No operation.
    NOP = 241,

    /// The data stream portion of a Synch. This should always be accompanied by a TCP Urgent notification.
    DataMark = 242,

    /// NVT character BRK.
    Break = 243,

    /// The function IP.
    InterruptProcess = 244,

    /// The function AO.
    AbortOutput = 245,

    /// The function AYT.
    AreYouThere = 246,

    /// The function EC.
    EraseCharacter = 247,

    /// The function EL.
    EraseLine = 248,

    /// The GA signal.
    GoAhead = 249,

    /// Indicates that what follows is subnegotiation of the indicated option.
    SB = 250,

    /// Indicates the desire to begin performing, or confirmation that
    /// you are now performing, the indicated option.
    WILL = 251,

    /// Indicates the refusal to perform, or continue performing, the indicated option.
    WONT = 252,
    
    /// Indicates the request that the other party perform, or confirmation that you are expecting
    /// the other party to perform, the indicated option.
    DO = 253,

    /// Indicates the demand that the other party stop performing, or confirmation that you are no,
    /// longer expecting the other party to perform, the indicated option.
    DONT = 254,

    /// Data Byte 255.
    IAC = 255
}