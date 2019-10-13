use cef_sys::{
    cef_browser_t, cef_errorcode_t, cef_frame_t, cef_load_handler_t,
    cef_string_t, cef_transition_type_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{collections::HashSet, convert::TryFrom};

use crate::{
    client::Client,
    browser::Browser,
    frame::Frame,
    refcounted::{RefCounted},
    string::CefString,
};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Any of the core values in [TransitionType] can be augmented by one or more qualifiers.
/// These qualifiers further define the transition.
pub enum TransitionTypeQualifiers {
    /// Attempted to visit a URL but was blocked.
    BlockedFlag = cef_transition_type_t::TT_BLOCKED_FLAG.0,
    /// Used the Forward or Back function to navigate among browsing history.
    ForwardBackFlag = cef_transition_type_t::TT_FORWARD_BACK_FLAG.0,
    /// The beginning of a navigation chain.
    ChainStartFlag = cef_transition_type_t::TT_CHAIN_START_FLAG.0,
    /// The last transition in a redirect chain.
    ChainEndFlag = cef_transition_type_t::TT_CHAIN_END_FLAG.0,
    /// Redirects caused by JavaScript or a meta refresh tag on the page.
    ClientRedirectFlag = cef_transition_type_t::TT_CLIENT_REDIRECT_FLAG.0,
    /// Redirects sent from the server by HTTP headers.
    ServerRedirectFlag = cef_transition_type_t::TT_SERVER_REDIRECT_FLAG.0,
}

impl TransitionTypeQualifiers {
    /// Used to test whether a transition involves a redirect.
    pub fn is_redirect(&self) -> bool {
        (*self as i32 & cef_transition_type_t::TT_IS_REDIRECT_MASK.0) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Transition type for a request. Made up of one source value and 0 or more
/// qualifiers.
pub enum TransitionType {
    /// Source is a link click or the JavaScript window.open function. This is
    /// also the default value for requests like sub-resource loads that are not
    /// navigations.
    Link(HashSet<TransitionTypeQualifiers>),
    /// Source is some other "explicit" navigation action such as creating a new
    /// browser or using the LoadURL function. This is also the default value
    /// for navigations where the actual type is unknown.
    Explicit(HashSet<TransitionTypeQualifiers>),
    /// Source is a subframe navigation. This is any content that is automatically
    /// loaded in a non-toplevel frame. For example, if a page consists of several
    /// frames containing ads, those ad URLs will have this transition type.
    /// The user may not even realize the content in these pages is a separate
    /// frame, so may not care about the URL.
    AutoSubframe(HashSet<TransitionTypeQualifiers>),
    /// Source is a subframe navigation explicitly requested by the user that will
    /// generate new navigation entries in the back/forward list. These are
    /// probably more important than frames that were automatically loaded in
    /// the background because the user probably cares about the fact that this
    /// link was loaded.
    ManualSubframe(HashSet<TransitionTypeQualifiers>),
    /// Source is a form submission by the user. NOTE: In some situations
    /// submitting a form does not result in this transition type. This can happen
    /// if the form uses a script to submit the contents.
    FormSubmit(HashSet<TransitionTypeQualifiers>),
    /// Source is a "reload" of the page via the Reload function or by re-visiting
    /// the same URL. NOTE: This is distinct from the concept of whether a
    /// particular load uses "reload semantics" (i.e. bypasses cached data).
    Reload(HashSet<TransitionTypeQualifiers>),
}

impl TryFrom<i32> for TransitionType {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let mut flags = HashSet::new();
        if value & cef_transition_type_t::TT_BLOCKED_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::BlockedFlag);
        }
        if value & cef_transition_type_t::TT_FORWARD_BACK_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::ForwardBackFlag);
        }
        if value & cef_transition_type_t::TT_CHAIN_START_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::ChainStartFlag);
        }
        if value & cef_transition_type_t::TT_CHAIN_END_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::ChainEndFlag);
        }
        if value & cef_transition_type_t::TT_CLIENT_REDIRECT_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::ClientRedirectFlag);
        }
        if value & cef_transition_type_t::TT_SERVER_REDIRECT_FLAG.0 != 0 {
            flags.insert(TransitionTypeQualifiers::ServerRedirectFlag);
        }
        match value & cef_transition_type_t::TT_SOURCE_MASK.0 {
            x if x == cef_transition_type_t::TT_LINK.0 => Ok(Self::Link(flags)),
            x if x == cef_transition_type_t::TT_EXPLICIT.0 => Ok(Self::Explicit(flags)),
            x if x == cef_transition_type_t::TT_AUTO_SUBFRAME.0 => Ok(Self::AutoSubframe(flags)),
            x if x == cef_transition_type_t::TT_MANUAL_SUBFRAME.0 => {
                Ok(Self::ManualSubframe(flags))
            }
            x if x == cef_transition_type_t::TT_FORM_SUBMIT.0 => Ok(Self::FormSubmit(flags)),
            x if x == cef_transition_type_t::TT_RELOAD.0 => Ok(Self::Reload(flags)),
            _ => Err(()),
        }
    }
}

impl Into<i32> for TransitionType {
    fn into(self) -> i32 {
        let value;
        let flags = match self {
            Self::Link(flags) => {
                value = cef_transition_type_t::TT_LINK;
                flags
            }
            Self::Explicit(flags) => {
                value = cef_transition_type_t::TT_EXPLICIT;
                flags
            }
            Self::AutoSubframe(flags) => {
                value = cef_transition_type_t::TT_AUTO_SUBFRAME;
                flags
            }
            Self::ManualSubframe(flags) => {
                value = cef_transition_type_t::TT_MANUAL_SUBFRAME;
                flags
            }
            Self::FormSubmit(flags) => {
                value = cef_transition_type_t::TT_FORM_SUBMIT;
                flags
            }
            Self::Reload(flags) => {
                value = cef_transition_type_t::TT_RELOAD;
                flags
            }
        };
        value.0 | flags.into_iter().fold(0, |flags, flag| flags | flag as i32)
    }
}

/// Supported error code values.
///
/// Ranges:
///     0- 99 System related errors
///   100-199 Connection related errors
///   200-299 Certificate errors
///   300-399 HTTP errors
///   400-499 Cache errors
///   500-599 ?
///   600-699 FTP errors
///   700-799 Certificate manager errors
///   800-899 DNS resolver errors
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, UnsafeFromPrimitive)]
pub enum ErrorCode { // this list is generated from cef_net_error_list.h using regex magic
    /// No error.
    None = cef_errorcode_t::ERR_NONE,

    /// An asynchronous IO operation is not yet complete.  This usually does not
    /// indicate a fatal error.  Typically this error will be generated as a
    /// notification to wait for some external notification that the IO operation
    /// finally completed.
    IoPending = cef_errorcode_t::ERR_IO_PENDING,

    /// A generic failure occurred.
    Failed = cef_errorcode_t::ERR_FAILED,

    /// An operation was aborted (due to user action).
    Aborted = cef_errorcode_t::ERR_ABORTED,

    /// An argument to the function is incorrect.
    InvalidArgument = cef_errorcode_t::ERR_INVALID_ARGUMENT,

    /// The handle or file descriptor is invalid.
    InvalidHandle = cef_errorcode_t::ERR_INVALID_HANDLE,

    /// The file or directory cannot be found.
    FileNotFound = cef_errorcode_t::ERR_FILE_NOT_FOUND,

    /// An operation timed out.
    TimedOut = cef_errorcode_t::ERR_TIMED_OUT,

    /// The file is too large.
    FileTooBig = cef_errorcode_t::ERR_FILE_TOO_BIG,

    /// An unexpected error.  This may be caused by a programming mistake or an
    /// invalid assumption.
    Unexpected = cef_errorcode_t::ERR_UNEXPECTED,

    /// Permission to access a resource, other than the network, was denied.
    AccessDenied = cef_errorcode_t::ERR_ACCESS_DENIED,

    /// The operation failed because of unimplemented functionality.
    NotImplemented = cef_errorcode_t::ERR_NOT_IMPLEMENTED,

    /// There were not enough resources to complete the operation.
    InsufficientResources = cef_errorcode_t::ERR_INSUFFICIENT_RESOURCES,

    /// Memory allocation failed.
    OutOfMemory = cef_errorcode_t::ERR_OUT_OF_MEMORY,

    /// The file upload failed because the file's modification time was different
    /// from the expectation.
    UploadFileChanged = cef_errorcode_t::ERR_UPLOAD_FILE_CHANGED,

    /// The socket is not connected.
    SocketNotConnected = cef_errorcode_t::ERR_SOCKET_NOT_CONNECTED,

    /// The file already exists.
    FileExists = cef_errorcode_t::ERR_FILE_EXISTS,

    /// The path or file name is too long.
    FilePathTooLong = cef_errorcode_t::ERR_FILE_PATH_TOO_LONG,

    /// Not enough room left on the disk.
    FileNoSpace = cef_errorcode_t::ERR_FILE_NO_SPACE,

    /// The file has a virus.
    FileVirusInfected = cef_errorcode_t::ERR_FILE_VIRUS_INFECTED,

    /// The client chose to block the request.
    BlockedByClient = cef_errorcode_t::ERR_BLOCKED_BY_CLIENT,

    /// The network changed.
    NetworkChanged = cef_errorcode_t::ERR_NETWORK_CHANGED,

    /// The request was blocked by the URL blacklist configured by the domain
    /// administrator.
    BlockedByAdministrator = cef_errorcode_t::ERR_BLOCKED_BY_ADMINISTRATOR,

    /// The socket is already connected.
    SocketIsConnected = cef_errorcode_t::ERR_SOCKET_IS_CONNECTED,

    /// The request was blocked because the forced reenrollment check is still
    /// pending. This error can only occur on ChromeOS.
    /// The error can be emitted by code in chrome/browser/policy/policy_helpers.cc.
    BlockedEnrollmentCheckPending = cef_errorcode_t::ERR_BLOCKED_ENROLLMENT_CHECK_PENDING,

    /// The upload failed because the upload stream needed to be re-read, due to a
    /// retry or a redirect, but the upload stream doesn't support that operation.
    UploadStreamRewindNotSupported = cef_errorcode_t::ERR_UPLOAD_STREAM_REWIND_NOT_SUPPORTED,

    /// The request failed because the URLRequestContext is shutting down, or has
    /// been shut down.
    ContextShutDown = cef_errorcode_t::ERR_CONTEXT_SHUT_DOWN,

    /// The request failed because the response was delivered along with requirements
    /// which are not met ('X-Frame-Options' and 'Content-Security-Policy' ancestor
    /// checks and 'Cross-Origin-Resource-Policy', for instance).
    BlockedByResponse = cef_errorcode_t::ERR_BLOCKED_BY_RESPONSE,

    /// The request failed after the response was received, based on client-side
    /// heuristics that point to the possiblility of a cross-site scripting attack.
    BlockedByXssAuditor = cef_errorcode_t::ERR_BLOCKED_BY_XSS_AUDITOR,

    /// The request was blocked by system policy disallowing some or all cleartext
    /// requests. Used for NetworkSecurityPolicy on Android.
    CleartextNotPermitted = cef_errorcode_t::ERR_CLEARTEXT_NOT_PERMITTED,

    /// A connection was closed (corresponding to a TCP FIN).
    ConnectionClosed = cef_errorcode_t::ERR_CONNECTION_CLOSED,

    /// A connection was reset (corresponding to a TCP RST).
    ConnectionReset = cef_errorcode_t::ERR_CONNECTION_RESET,

    /// A connection attempt was refused.
    ConnectionRefused = cef_errorcode_t::ERR_CONNECTION_REFUSED,

    /// A connection timed out as a result of not receiving an ACK for data sent.
    /// This can include a FIN packet that did not get ACK'd.
    ConnectionAborted = cef_errorcode_t::ERR_CONNECTION_ABORTED,

    /// A connection attempt failed.
    ConnectionFailed = cef_errorcode_t::ERR_CONNECTION_FAILED,

    /// The host name could not be resolved.
    NameNotResolved = cef_errorcode_t::ERR_NAME_NOT_RESOLVED,

    /// The Internet connection has been lost.
    InternetDisconnected = cef_errorcode_t::ERR_INTERNET_DISCONNECTED,

    /// An SSL protocol error occurred.
    SslProtocolError = cef_errorcode_t::ERR_SSL_PROTOCOL_ERROR,

    /// The IP address or port number is invalid (e.g., cannot connect to the IP
    /// address 0 or the port 0).
    AddressInvalid = cef_errorcode_t::ERR_ADDRESS_INVALID,

    /// The IP address is unreachable.  This usually means that there is no route to
    /// the specified host or network.
    AddressUnreachable = cef_errorcode_t::ERR_ADDRESS_UNREACHABLE,

    /// The server requested a client certificate for SSL client authentication.
    SslClientAuthCertNeeded = cef_errorcode_t::ERR_SSL_CLIENT_AUTH_CERT_NEEDED,

    /// A tunnel connection through the proxy could not be established.
    TunnelConnectionFailed = cef_errorcode_t::ERR_TUNNEL_CONNECTION_FAILED,

    /// No SSL protocol versions are enabled.
    NoSslVersionsEnabled = cef_errorcode_t::ERR_NO_SSL_VERSIONS_ENABLED,

    /// The client and server don't support a common SSL protocol version or
    /// cipher suite.
    SslVersionOrCipherMismatch = cef_errorcode_t::ERR_SSL_VERSION_OR_CIPHER_MISMATCH,

    /// The server requested a renegotiation (rehandshake).
    SslRenegotiationRequested = cef_errorcode_t::ERR_SSL_RENEGOTIATION_REQUESTED,

    /// The proxy requested authentication (for tunnel establishment) with an
    /// unsupported method.
    ProxyAuthUnsupported = cef_errorcode_t::ERR_PROXY_AUTH_UNSUPPORTED,

    /// During SSL renegotiation (rehandshake), the server sent a certificate with
    /// an error.
    ///
    /// Note: this error is not in the -2xx range so that it won't be handled as a
    /// certificate error.
    CertErrorInSslRenegotiation = cef_errorcode_t::ERR_CERT_ERROR_IN_SSL_RENEGOTIATION,

    /// The SSL handshake failed because of a bad or missing client certificate.
    BadSslClientAuthCert = cef_errorcode_t::ERR_BAD_SSL_CLIENT_AUTH_CERT,

    /// A connection attempt timed out.
    ConnectionTimedOut = cef_errorcode_t::ERR_CONNECTION_TIMED_OUT,

    /// There are too many pending DNS resolves, so a request in the queue was
    /// aborted.
    HostResolverQueueTooLarge = cef_errorcode_t::ERR_HOST_RESOLVER_QUEUE_TOO_LARGE,

    /// Failed establishing a connection to the SOCKS proxy server for a target host.
    SocksConnectionFailed = cef_errorcode_t::ERR_SOCKS_CONNECTION_FAILED,

    /// The SOCKS proxy server failed establishing connection to the target host
    /// because that host is unreachable.
    SocksConnectionHostUnreachable = cef_errorcode_t::ERR_SOCKS_CONNECTION_HOST_UNREACHABLE,

    /// The request to negotiate an alternate protocol failed.
    AlpnNegotiationFailed = cef_errorcode_t::ERR_ALPN_NEGOTIATION_FAILED,

    /// The peer sent an SSL no_renegotiation alert message.
    SslNoRenegotiation = cef_errorcode_t::ERR_SSL_NO_RENEGOTIATION,

    /// Winsock sometimes reports more data written than passed.  This is probably
    /// due to a broken LSP.
    WinsockUnexpectedWrittenBytes = cef_errorcode_t::ERR_WINSOCK_UNEXPECTED_WRITTEN_BYTES,

    /// An SSL peer sent us a fatal decompression_failure alert. This typically
    /// occurs when a peer selects DEFLATE compression in the mistaken belief that
    /// it supports it.
    SslDecompressionFailureAlert = cef_errorcode_t::ERR_SSL_DECOMPRESSION_FAILURE_ALERT,

    /// An SSL peer sent us a fatal bad_record_mac alert. This has been observed
    /// from servers with buggy DEFLATE support.
    SslBadRecordMacAlert = cef_errorcode_t::ERR_SSL_BAD_RECORD_MAC_ALERT,

    /// The proxy requested authentication (for tunnel establishment).
    ProxyAuthRequested = cef_errorcode_t::ERR_PROXY_AUTH_REQUESTED,

    /// The SSL server attempted to use a weak ephemeral Diffie-Hellman key.
    SslWeakServerEphemeralDhKey = cef_errorcode_t::ERR_SSL_WEAK_SERVER_EPHEMERAL_DH_KEY,

    /// Could not create a connection to the proxy server. An error occurred
    /// either in resolving its name, or in connecting a socket to it.
    /// Note that this does NOT include failures during the actual "CONNECT" method
    /// of an HTTP proxy.
    ProxyConnectionFailed = cef_errorcode_t::ERR_PROXY_CONNECTION_FAILED,

    /// A mandatory proxy configuration could not be used. Currently this means
    /// that a mandatory PAC script could not be fetched, parsed or executed.
    MandatoryProxyConfigurationFailed = cef_errorcode_t::ERR_MANDATORY_PROXY_CONFIGURATION_FAILED,

    /// -132 was formerly ERR_ESET_ANTI_VIRUS_SSL_INTERCEPTION

    /// We've hit the max socket limit for the socket pool while preconnecting.  We
    /// don't bother trying to preconnect more sockets.
    PreconnectMaxSocketLimit = cef_errorcode_t::ERR_PRECONNECT_MAX_SOCKET_LIMIT,

    /// The permission to use the SSL client certificate's private key was denied.
    SslClientAuthPrivateKeyAccessDenied =
        cef_errorcode_t::ERR_SSL_CLIENT_AUTH_PRIVATE_KEY_ACCESS_DENIED,

    /// The SSL client certificate has no private key.
    SslClientAuthCertNoPrivateKey = cef_errorcode_t::ERR_SSL_CLIENT_AUTH_CERT_NO_PRIVATE_KEY,

    /// The certificate presented by the HTTPS Proxy was invalid.
    ProxyCertificateInvalid = cef_errorcode_t::ERR_PROXY_CERTIFICATE_INVALID,

    /// An error occurred when trying to do a name resolution (DNS).
    NameResolutionFailed = cef_errorcode_t::ERR_NAME_RESOLUTION_FAILED,

    /// Permission to access the network was denied. This is used to distinguish
    /// errors that were most likely caused by a firewall from other access denied
    /// errors. See also ERR_ACCESS_DENIED.
    NetworkAccessDenied = cef_errorcode_t::ERR_NETWORK_ACCESS_DENIED,

    /// The request throttler module cancelled this request to avoid DDOS.
    TemporarilyThrottled = cef_errorcode_t::ERR_TEMPORARILY_THROTTLED,

    /// We were unable to sign the CertificateVerify data of an SSL client auth
    /// handshake with the client certificate's private key.
    ///
    /// Possible causes for this include the user implicitly or explicitly
    /// denying access to the private key, the private key may not be valid for
    /// signing, the key may be relying on a cached handle which is no longer
    /// valid, or the CSP won't allow arbitrary data to be signed.
    SslClientAuthSignatureFailed = cef_errorcode_t::ERR_SSL_CLIENT_AUTH_SIGNATURE_FAILED,

    /// The message was too large for the transport.  (for example a UDP message
    /// which exceeds size threshold).
    MsgTooBig = cef_errorcode_t::ERR_MSG_TOO_BIG,

    /// Websocket protocol error. Indicates that we are terminating the connection
    /// due to a malformed frame or other protocol violation.
    WsProtocolError = cef_errorcode_t::ERR_WS_PROTOCOL_ERROR,

    /// Returned when attempting to bind an address that is already in use.
    AddressInUse = cef_errorcode_t::ERR_ADDRESS_IN_USE,

    /// An operation failed because the SSL handshake has not completed.
    SslHandshakeNotCompleted = cef_errorcode_t::ERR_SSL_HANDSHAKE_NOT_COMPLETED,

    /// SSL peer's public key is invalid.
    SslBadPeerPublicKey = cef_errorcode_t::ERR_SSL_BAD_PEER_PUBLIC_KEY,

    /// The certificate didn't match the built-in public key pins for the host name.
    /// The pins are set in net/http/transport_security_state.cc and require that
    /// one of a set of public keys exist on the path from the leaf to the root.
    SslPinnedKeyNotInCertChain = cef_errorcode_t::ERR_SSL_PINNED_KEY_NOT_IN_CERT_CHAIN,

    /// Server request for client certificate did not contain any types we support.
    ClientAuthCertTypeUnsupported = cef_errorcode_t::ERR_CLIENT_AUTH_CERT_TYPE_UNSUPPORTED,

    /// An SSL peer sent us a fatal decrypt_error alert. This typically occurs when
    /// a peer could not correctly verify a signature (in CertificateVerify or
    /// ServerKeyExchange) or validate a Finished message.
    SslDecryptErrorAlert = cef_errorcode_t::ERR_SSL_DECRYPT_ERROR_ALERT,

    /// There are too many pending WebSocketJob instances, so the new job was not
    /// pushed to the queue.
    WsThrottleQueueTooLarge = cef_errorcode_t::ERR_WS_THROTTLE_QUEUE_TOO_LARGE,

    /// The SSL server certificate changed in a renegotiation.
    SslServerCertChanged = cef_errorcode_t::ERR_SSL_SERVER_CERT_CHANGED,

    /// The SSL server sent us a fatal unrecognized_name alert.
    SslUnrecognizedNameAlert = cef_errorcode_t::ERR_SSL_UNRECOGNIZED_NAME_ALERT,

    /// Failed to set the socket's receive buffer size as requested.
    SocketSetReceiveBufferSizeError = cef_errorcode_t::ERR_SOCKET_SET_RECEIVE_BUFFER_SIZE_ERROR,

    /// Failed to set the socket's send buffer size as requested.
    SocketSetSendBufferSizeError = cef_errorcode_t::ERR_SOCKET_SET_SEND_BUFFER_SIZE_ERROR,

    /// Failed to set the socket's receive buffer size as requested, despite success
    /// return code from setsockopt.
    SocketReceiveBufferSizeUnchangeable =
        cef_errorcode_t::ERR_SOCKET_RECEIVE_BUFFER_SIZE_UNCHANGEABLE,

    /// Failed to set the socket's send buffer size as requested, despite success
    /// return code from setsockopt.
    SocketSendBufferSizeUnchangeable = cef_errorcode_t::ERR_SOCKET_SEND_BUFFER_SIZE_UNCHANGEABLE,

    /// Failed to import a client certificate from the platform store into the SSL
    /// library.
    SslClientAuthCertBadFormat = cef_errorcode_t::ERR_SSL_CLIENT_AUTH_CERT_BAD_FORMAT,

    /// Resolving a hostname to an IP address list included the IPv4 address
    /// "127.0.53.53". This is a special IP address which ICANN has recommended to
    /// indicate there was a name collision, and alert admins to a potential
    /// problem.
    IcannNameCollision = cef_errorcode_t::ERR_ICANN_NAME_COLLISION,

    /// The SSL server presented a certificate which could not be decoded. This is
    /// not a certificate error code as no X509Certificate object is available. This
    /// error is fatal.
    SslServerCertBadFormat = cef_errorcode_t::ERR_SSL_SERVER_CERT_BAD_FORMAT,

    /// Certificate Transparency: Received a signed tree head that failed to parse.
    CtSthParsingFailed = cef_errorcode_t::ERR_CT_STH_PARSING_FAILED,

    /// Certificate Transparency: Received a signed tree head whose JSON parsing was
    /// OK but was missing some of the fields.
    CtSthIncomplete = cef_errorcode_t::ERR_CT_STH_INCOMPLETE,

    /// The attempt to reuse a connection to send proxy auth credentials failed
    /// before the AuthController was used to generate credentials. The caller should
    /// reuse the controller with a new connection. This error is only used
    /// internally by the network stack.
    UnableToReuseConnectionForProxyAuth =
        cef_errorcode_t::ERR_UNABLE_TO_REUSE_CONNECTION_FOR_PROXY_AUTH,

    /// Certificate Transparency: Failed to parse the received consistency proof.
    CtConsistencyProofParsingFailed = cef_errorcode_t::ERR_CT_CONSISTENCY_PROOF_PARSING_FAILED,

    /// The SSL server required an unsupported cipher suite that has since been
    /// removed. This error will temporarily be signaled on a fallback for one or two
    /// releases immediately following a cipher suite's removal, after which the
    /// fallback will be removed.
    SslObsoleteCipher = cef_errorcode_t::ERR_SSL_OBSOLETE_CIPHER,

    /// When a WebSocket handshake is done successfully and the connection has been
    /// upgraded, the URLRequest is cancelled with this error code.
    WsUpgrade = cef_errorcode_t::ERR_WS_UPGRADE,

    /// Socket ReadIfReady support is not implemented. This error should not be user
    /// visible, because the normal Read() method is used as a fallback.
    ReadIfReadyNotImplemented = cef_errorcode_t::ERR_READ_IF_READY_NOT_IMPLEMENTED,

    /// No socket buffer space is available.
    NoBufferSpace = cef_errorcode_t::ERR_NO_BUFFER_SPACE,

    /// There were no common signature algorithms between our client certificate
    /// private key and the server's preferences.
    SslClientAuthNoCommonAlgorithms = cef_errorcode_t::ERR_SSL_CLIENT_AUTH_NO_COMMON_ALGORITHMS,

    /// TLS 1.3 early data was rejected by the server. This will be received before
    /// any data is returned from the socket. The request should be retried with
    /// early data disabled.
    EarlyDataRejected = cef_errorcode_t::ERR_EARLY_DATA_REJECTED,

    /// TLS 1.3 early data was offered, but the server responded with TLS 1.2 or
    /// earlier. This is an internal error code to account for a
    /// backwards-compatibility issue with early data and TLS 1.2. It will be
    /// received before any data is returned from the socket. The request should be
    /// retried with early data disabled.
    ///
    /// See https:///tools.ietf.org/html/rfc8446#appendix-D.3 for details.
    WrongVersionOnEarlyData = cef_errorcode_t::ERR_WRONG_VERSION_ON_EARLY_DATA,

    /// TLS 1.3 was enabled, but a lower version was negotiated and the server
    /// returned a value indicating it supported TLS 1.3. This is part of a security
    /// check in TLS 1.3, but it may also indicate the user is behind a buggy
    /// TLS-terminating proxy which implemented TLS 1.2 incorrectly. (See
    /// https:///crbug.com/boringssl/226.)
    Tls13DowngradeDetected = cef_errorcode_t::ERR_TLS13_DOWNGRADE_DETECTED,

    /// The server's certificate has a keyUsage extension incompatible with the
    /// negotiated TLS key exchange method.
    SslKeyUsageIncompatible = cef_errorcode_t::ERR_SSL_KEY_USAGE_INCOMPATIBLE,

    /// The server responded with a certificate whose common name did not match
    /// the host name.  This could mean:
    ///
    /// 1. An attacker has redirected our traffic to their server and is
    ///    presenting a certificate for which they know the private key.
    ///
    /// 2. The server is misconfigured and responding with the wrong cert.
    ///
    /// 3. The user is on a wireless network and is being redirected to the
    ///    network's login page.
    ///
    /// 4. The OS has used a DNS search suffix and the server doesn't have
    ///    a certificate for the abbreviated name in the address bar.
    ///
    CertCommonNameInvalid = cef_errorcode_t::ERR_CERT_COMMON_NAME_INVALID,

    /// The server responded with a certificate that, by our clock, appears to
    /// either not yet be valid or to have expired.  This could mean:
    ///
    /// 1. An attacker is presenting an old certificate for which they have
    ///    managed to obtain the private key.
    ///
    /// 2. The server is misconfigured and is not presenting a valid cert.
    ///
    /// 3. Our clock is wrong.
    ///
    CertDateInvalid = cef_errorcode_t::ERR_CERT_DATE_INVALID,

    /// The server responded with a certificate that is signed by an authority
    /// we don't trust.  The could mean:
    ///
    /// 1. An attacker has substituted the real certificate for a cert that
    ///    contains their public key and is signed by their cousin.
    ///
    /// 2. The server operator has a legitimate certificate from a CA we don't
    ///    know about, but should trust.
    ///
    /// 3. The server is presenting a self-signed certificate, providing no
    ///    defense against active attackers (but foiling passive attackers).
    ///
    CertAuthorityInvalid = cef_errorcode_t::ERR_CERT_AUTHORITY_INVALID,

    /// The server responded with a certificate that contains errors.
    /// This error is not recoverable.
    ///
    /// MSDN describes this error as follows:
    ///   "The SSL certificate contains errors."
    /// NOTE: It's unclear how this differs from ERR_CERT_INVALID. For consistency,
    /// use that code instead of this one from now on.
    ///
    CertContainsErrors = cef_errorcode_t::ERR_CERT_CONTAINS_ERRORS,

    /// The certificate has no mechanism for determining if it is revoked.  In
    /// effect, this certificate cannot be revoked.
    CertNoRevocationMechanism = cef_errorcode_t::ERR_CERT_NO_REVOCATION_MECHANISM,

    /// Revocation information for the security certificate for this site is not
    /// available.  This could mean:
    ///
    /// 1. An attacker has compromised the private key in the certificate and is
    ///    blocking our attempt to find out that the cert was revoked.
    ///
    /// 2. The certificate is unrevoked, but the revocation server is busy or
    ///    unavailable.
    ///
    CertUnableToCheckRevocation = cef_errorcode_t::ERR_CERT_UNABLE_TO_CHECK_REVOCATION,

    /// The server responded with a certificate has been revoked.
    /// We have the capability to ignore this error, but it is probably not the
    /// thing to do.
    CertRevoked = cef_errorcode_t::ERR_CERT_REVOKED,

    /// The server responded with a certificate that is invalid.
    /// This error is not recoverable.
    ///
    /// MSDN describes this error as follows:
    ///   "The SSL certificate is invalid."
    ///
    CertInvalid = cef_errorcode_t::ERR_CERT_INVALID,

    /// The server responded with a certificate that is signed using a weak
    /// signature algorithm.
    CertWeakSignatureAlgorithm = cef_errorcode_t::ERR_CERT_WEAK_SIGNATURE_ALGORITHM,

    /// The host name specified in the certificate is not unique.
    CertNonUniqueName = cef_errorcode_t::ERR_CERT_NON_UNIQUE_NAME,

    /// The server responded with a certificate that contains a weak key (e.g.
    /// a too-small RSA key).
    CertWeakKey = cef_errorcode_t::ERR_CERT_WEAK_KEY,

    /// The certificate claimed DNS names that are in violation of name constraints.
    CertNameConstraintViolation = cef_errorcode_t::ERR_CERT_NAME_CONSTRAINT_VIOLATION,

    /// The certificate's validity period is too long.
    CertValidityTooLong = cef_errorcode_t::ERR_CERT_VALIDITY_TOO_LONG,

    /// Certificate Transparency was required for this connection, but the server
    /// did not provide CT information that complied with the policy.
    CertificateTransparencyRequired = cef_errorcode_t::ERR_CERTIFICATE_TRANSPARENCY_REQUIRED,

    /// The certificate chained to a legacy Symantec root that is no longer trusted.
    /// https:///g.co/chrome/symantecpkicerts
    CertSymantecLegacy = cef_errorcode_t::ERR_CERT_SYMANTEC_LEGACY,

    /// The URL is invalid.
    InvalidUrl = cef_errorcode_t::ERR_INVALID_URL,

    /// The scheme of the URL is disallowed.
    DisallowedUrlScheme = cef_errorcode_t::ERR_DISALLOWED_URL_SCHEME,

    /// The scheme of the URL is unknown.
    UnknownUrlScheme = cef_errorcode_t::ERR_UNKNOWN_URL_SCHEME,

    /// Attempting to load an URL resulted in a redirect to an invalid URL.
    InvalidRedirect = cef_errorcode_t::ERR_INVALID_REDIRECT,

    /// Attempting to load an URL resulted in too many redirects.
    TooManyRedirects = cef_errorcode_t::ERR_TOO_MANY_REDIRECTS,

    /// Attempting to load an URL resulted in an unsafe redirect (e.g., a redirect
    /// to file:/// is considered unsafe).
    UnsafeRedirect = cef_errorcode_t::ERR_UNSAFE_REDIRECT,

    /// Attempting to load an URL with an unsafe port number.  These are port
    /// numbers that correspond to services, which are not robust to spurious input
    /// that may be constructed as a result of an allowed web construct (e.g., HTTP
    /// looks a lot like SMTP, so form submission to port 25 is denied).
    UnsafePort = cef_errorcode_t::ERR_UNSAFE_PORT,

    /// The server's response was invalid.
    InvalidResponse = cef_errorcode_t::ERR_INVALID_RESPONSE,

    /// Error in chunked transfer encoding.
    InvalidChunkedEncoding = cef_errorcode_t::ERR_INVALID_CHUNKED_ENCODING,

    /// The server did not support the request method.
    MethodNotSupported = cef_errorcode_t::ERR_METHOD_NOT_SUPPORTED,

    /// The response was 407 (Proxy Authentication Required), yet we did not send
    /// the request to a proxy.
    UnexpectedProxyAuth = cef_errorcode_t::ERR_UNEXPECTED_PROXY_AUTH,

    /// The server closed the connection without sending any data.
    EmptyResponse = cef_errorcode_t::ERR_EMPTY_RESPONSE,

    /// The headers section of the response is too large.
    ResponseHeadersTooBig = cef_errorcode_t::ERR_RESPONSE_HEADERS_TOO_BIG,

    /// The PAC requested by HTTP did not have a valid status code (non-200).
    PacStatusNotOk = cef_errorcode_t::ERR_PAC_STATUS_NOT_OK,

    /// The evaluation of the PAC script failed.
    PacScriptFailed = cef_errorcode_t::ERR_PAC_SCRIPT_FAILED,

    /// The response was 416 (Requested range not satisfiable) and the server cannot
    /// satisfy the range requested.
    RequestRangeNotSatisfiable = cef_errorcode_t::ERR_REQUEST_RANGE_NOT_SATISFIABLE,

    /// The identity used for authentication is invalid.
    MalformedIdentity = cef_errorcode_t::ERR_MALFORMED_IDENTITY,

    /// Content decoding of the response body failed.
    ContentDecodingFailed = cef_errorcode_t::ERR_CONTENT_DECODING_FAILED,

    /// An operation could not be completed because all network IO
    /// is suspended.
    NetworkIoSuspended = cef_errorcode_t::ERR_NETWORK_IO_SUSPENDED,

    /// FLIP data received without receiving a SYN_REPLY on the stream.
    SynReplyNotReceived = cef_errorcode_t::ERR_SYN_REPLY_NOT_RECEIVED,

    /// Converting the response to target encoding failed.
    EncodingConversionFailed = cef_errorcode_t::ERR_ENCODING_CONVERSION_FAILED,

    /// The server sent an FTP directory listing in a format we do not understand.
    UnrecognizedFtpDirectoryListingFormat =
        cef_errorcode_t::ERR_UNRECOGNIZED_FTP_DIRECTORY_LISTING_FORMAT,

    /// There are no supported proxies in the provided list.
    NoSupportedProxies = cef_errorcode_t::ERR_NO_SUPPORTED_PROXIES,

    /// There is a SPDY protocol error.
    Http2ProtocolError = cef_errorcode_t::ERR_HTTP2_PROTOCOL_ERROR,

    /// Credentials could not be established during HTTP Authentication.
    InvalidAuthCredentials = cef_errorcode_t::ERR_INVALID_AUTH_CREDENTIALS,

    /// An HTTP Authentication scheme was tried which is not supported on this
    /// machine.
    UnsupportedAuthScheme = cef_errorcode_t::ERR_UNSUPPORTED_AUTH_SCHEME,

    /// Detecting the encoding of the response failed.
    EncodingDetectionFailed = cef_errorcode_t::ERR_ENCODING_DETECTION_FAILED,

    /// (GSSAPI) No Kerberos credentials were available during HTTP Authentication.
    MissingAuthCredentials = cef_errorcode_t::ERR_MISSING_AUTH_CREDENTIALS,

    /// An unexpected, but documented, SSPI or GSSAPI status code was returned.
    UnexpectedSecurityLibraryStatus = cef_errorcode_t::ERR_UNEXPECTED_SECURITY_LIBRARY_STATUS,

    /// The environment was not set up correctly for authentication (for
    /// example, no KDC could be found or the principal is unknown.
    MisconfiguredAuthEnvironment = cef_errorcode_t::ERR_MISCONFIGURED_AUTH_ENVIRONMENT,

    /// An undocumented SSPI or GSSAPI status code was returned.
    UndocumentedSecurityLibraryStatus = cef_errorcode_t::ERR_UNDOCUMENTED_SECURITY_LIBRARY_STATUS,

    /// The HTTP response was too big to drain.
    ResponseBodyTooBigToDrain = cef_errorcode_t::ERR_RESPONSE_BODY_TOO_BIG_TO_DRAIN,

    /// The HTTP response contained multiple distinct Content-Length headers.
    ResponseHeadersMultipleContentLength =
        cef_errorcode_t::ERR_RESPONSE_HEADERS_MULTIPLE_CONTENT_LENGTH,

    /// HTTP/2 headers have been received, but not all of them - status or version
    /// headers are missing, so we're expecting additional frames to complete them.
    IncompleteHttp2Headers = cef_errorcode_t::ERR_INCOMPLETE_HTTP2_HEADERS,

    /// No PAC URL configuration could be retrieved from DHCP. This can indicate
    /// either a failure to retrieve the DHCP configuration, or that there was no
    /// PAC URL configured in DHCP.
    PACNotInDHCP = cef_errorcode_t::ERR_PAC_NOT_IN_DHCP,

    /// The HTTP response contained multiple Content-Disposition headers.
    ResponseHeadersMultipleContentDisposition =
        cef_errorcode_t::ERR_RESPONSE_HEADERS_MULTIPLE_CONTENT_DISPOSITION,

    /// The HTTP response contained multiple Location headers.
    ResponseHeadersMultipleLocation = cef_errorcode_t::ERR_RESPONSE_HEADERS_MULTIPLE_LOCATION,

    /// HTTP/2 server refused the request without processing, and sent either a
    /// GOAWAY frame with error code NO_ERROR and Last-Stream-ID lower than the
    /// stream id corresponding to the request indicating that this request has not
    /// been processed yet, or a RST_STREAM frame with error code REFUSED_STREAM.
    /// Client MAY retry (on a different connection).  See RFC7540 Section 8.1.4.
    Http2ServerRefusedStream = cef_errorcode_t::ERR_HTTP2_SERVER_REFUSED_STREAM,

    /// HTTP/2 server didn't respond to the PING message.
    Http2PingFailed = cef_errorcode_t::ERR_HTTP2_PING_FAILED,

    /// The HTTP response body transferred fewer bytes than were advertised by the
    /// Content-Length header when the connection is closed.
    ContentLengthMismatch = cef_errorcode_t::ERR_CONTENT_LENGTH_MISMATCH,

    /// The HTTP response body is transferred with Chunked-Encoding, but the
    /// terminating zero-length chunk was never sent when the connection is closed.
    IncompleteChunkedEncoding = cef_errorcode_t::ERR_INCOMPLETE_CHUNKED_ENCODING,

    /// There is a QUIC protocol error.
    QuicProtocolError = cef_errorcode_t::ERR_QUIC_PROTOCOL_ERROR,

    /// The HTTP headers were truncated by an EOF.
    ResponseHeadersTruncated = cef_errorcode_t::ERR_RESPONSE_HEADERS_TRUNCATED,

    /// The QUIC crytpo handshake failed.  This means that the server was unable
    /// to read any requests sent, so they may be resent.
    QuicHandshakeFailed = cef_errorcode_t::ERR_QUIC_HANDSHAKE_FAILED,

    /// Transport security is inadequate for the HTTP/2 version.
    Http2InadequateTransportSecurity = cef_errorcode_t::ERR_HTTP2_INADEQUATE_TRANSPORT_SECURITY,

    /// The peer violated HTTP/2 flow control.
    Http2FlowControlError = cef_errorcode_t::ERR_HTTP2_FLOW_CONTROL_ERROR,

    /// The peer sent an improperly sized HTTP/2 frame.
    Http2FrameSizeError = cef_errorcode_t::ERR_HTTP2_FRAME_SIZE_ERROR,

    /// Decoding or encoding of compressed HTTP/2 headers failed.
    Http2CompressionError = cef_errorcode_t::ERR_HTTP2_COMPRESSION_ERROR,

    /// Proxy Auth Requested without a valid Client Socket Handle.
    ProxyAuthRequestedWithNoConnection = cef_errorcode_t::ERR_PROXY_AUTH_REQUESTED_WITH_NO_CONNECTION,

    /// HTTP_1_1_REQUIRED error code received on HTTP/2 session.
    Http11Required = cef_errorcode_t::ERR_HTTP_1_1_REQUIRED,

    /// HTTP_1_1_REQUIRED error code received on HTTP/2 session to proxy.
    ProxyHttp11Required = cef_errorcode_t::ERR_PROXY_HTTP_1_1_REQUIRED,

    /// The PAC script terminated fatally and must be reloaded.
    PacScriptTerminated = cef_errorcode_t::ERR_PAC_SCRIPT_TERMINATED,

    /// The server was expected to return an HTTP/1.x response, but did not. Rather
    /// than treat it as HTTP/0.9, this error is returned.
    InvalidHttpResponse = cef_errorcode_t::ERR_INVALID_HTTP_RESPONSE,

    /// Initializing content decoding failed.
    ContentDecodingInitFailed = cef_errorcode_t::ERR_CONTENT_DECODING_INIT_FAILED,

    /// Received HTTP/2 RST_STREAM frame with NO_ERROR error code.  This error should
    /// be handled internally by HTTP/2 code, and should not make it above the
    /// SpdyStream layer.
    Http2RstStreamNoErrorReceived = cef_errorcode_t::ERR_HTTP2_RST_STREAM_NO_ERROR_RECEIVED,

    /// The pushed stream claimed by the request is no longer available.
    Http2PushedStreamNotAvailable = cef_errorcode_t::ERR_HTTP2_PUSHED_STREAM_NOT_AVAILABLE,

    /// A pushed stream was claimed and later reset by the server. When this happens,
    /// the request should be retried.
    Http2ClaimedPushedStreamResetByServer = cef_errorcode_t::ERR_HTTP2_CLAIMED_PUSHED_STREAM_RESET_BY_SERVER,

    /// An HTTP transaction was retried too many times due for authentication or
    /// invalid certificates. This may be due to a bug in the net stack that would
    /// otherwise infinite loop, or if the server or proxy continually requests fresh
    /// credentials or presents a fresh invalid certificate.
    TooManyRetries = cef_errorcode_t::ERR_TOO_MANY_RETRIES,

    /// Received an HTTP/2 frame on a closed stream.
    Http2StreamClosed = cef_errorcode_t::ERR_HTTP2_STREAM_CLOSED,

    /// Client is refusing an HTTP/2 stream.
    Http2ClientRefusedStream = cef_errorcode_t::ERR_HTTP2_CLIENT_REFUSED_STREAM,

    /// A pushed HTTP/2 stream was claimed by a request based on matching URL and
    /// request headers, but the pushed response headers do not match the request.
    Http2PushedResponseDoesNotMatch = cef_errorcode_t::ERR_HTTP2_PUSHED_RESPONSE_DOES_NOT_MATCH,

    /// The cache does not have the requested entry.
    CacheMiss = cef_errorcode_t::ERR_CACHE_MISS,

    /// Unable to read from the disk cache.
    CacheReadFailure = cef_errorcode_t::ERR_CACHE_READ_FAILURE,

    /// Unable to write to the disk cache.
    CacheWriteFailure = cef_errorcode_t::ERR_CACHE_WRITE_FAILURE,

    /// The operation is not supported for this entry.
    CacheOperationNotSupported = cef_errorcode_t::ERR_CACHE_OPERATION_NOT_SUPPORTED,

    /// The disk cache is unable to open this entry.
    CacheOpenFailure = cef_errorcode_t::ERR_CACHE_OPEN_FAILURE,

    /// The disk cache is unable to create this entry.
    CacheCreateFailure = cef_errorcode_t::ERR_CACHE_CREATE_FAILURE,

    /// Multiple transactions are racing to create disk cache entries. This is an
    /// internal error returned from the HttpCache to the HttpCacheTransaction that
    /// tells the transaction to restart the entry-creation logic because the state
    /// of the cache has changed.
    CacheRace = cef_errorcode_t::ERR_CACHE_RACE,

    /// The cache was unable to read a checksum record on an entry. This can be
    /// returned from attempts to read from the cache. It is an internal error,
    /// returned by the SimpleCache backend, but not by any URLRequest methods
    /// or members.
    CacheChecksumReadFailure = cef_errorcode_t::ERR_CACHE_CHECKSUM_READ_FAILURE,

    /// The cache found an entry with an invalid checksum. This can be returned from
    /// attempts to read from the cache. It is an internal error, returned by the
    /// SimpleCache backend, but not by any URLRequest methods or members.
    CacheChecksumMismatch = cef_errorcode_t::ERR_CACHE_CHECKSUM_MISMATCH,

    /// Internal error code for the HTTP cache. The cache lock timeout has fired.
    CacheLockTimeout = cef_errorcode_t::ERR_CACHE_LOCK_TIMEOUT,

    /// Received a challenge after the transaction has read some data, and the
    /// credentials aren't available.  There isn't a way to get them at that point.
    CacheAuthFailureAfterRead = cef_errorcode_t::ERR_CACHE_AUTH_FAILURE_AFTER_READ,

    /// Internal not-quite error code for the HTTP cache. In-memory hints suggest
    /// that the cache entry would not have been useable with the transaction's
    /// current configuration (e.g. load flags, mode, etc.)
    CacheEntryNotSuitable = cef_errorcode_t::ERR_CACHE_ENTRY_NOT_SUITABLE,

    /// The disk cache is unable to doom this entry.
    CacheDoomFailure = cef_errorcode_t::ERR_CACHE_DOOM_FAILURE,

    /// The disk cache is unable to open or create this entry.
    CacheOpenOrCreateFailure = cef_errorcode_t::ERR_CACHE_OPEN_OR_CREATE_FAILURE,

    /// The server's response was insecure (e.g. there was a cert error).
    InsecureResponse = cef_errorcode_t::ERR_INSECURE_RESPONSE,

    /// An attempt to import a client certificate failed, as the user's key
    /// database lacked a corresponding private key.
    NoPrivateKeyForCert = cef_errorcode_t::ERR_NO_PRIVATE_KEY_FOR_CERT,

    /// An error adding a certificate to the OS certificate database.
    AddUserCertFailed = cef_errorcode_t::ERR_ADD_USER_CERT_FAILED,

    /// An error occurred while handling a signed exchange.
    InvalidSignedExchange = cef_errorcode_t::ERR_INVALID_SIGNED_EXCHANGE,

    /// A generic error for failed FTP control connection command.
    /// If possible, please use or add a more specific error code.
    FtpFailed = cef_errorcode_t::ERR_FTP_FAILED,

    /// The server cannot fulfill the request at this point. This is a temporary
    /// error.
    /// FTP response code 421.
    FtpServiceUnavailable = cef_errorcode_t::ERR_FTP_SERVICE_UNAVAILABLE,

    /// The server has aborted the transfer.
    /// FTP response code 426.
    FtpTransferAborted = cef_errorcode_t::ERR_FTP_TRANSFER_ABORTED,

    /// The file is busy, or some other temporary error condition on opening
    /// the file.
    /// FTP response code 450.
    FtpFileBusy = cef_errorcode_t::ERR_FTP_FILE_BUSY,

    /// Server rejected our command because of syntax errors.
    /// FTP response codes 500, 501.
    FtpSyntaxError = cef_errorcode_t::ERR_FTP_SYNTAX_ERROR,

    /// Server does not support the command we issued.
    /// FTP response codes 502, 504.
    FtpCommandNotSupported = cef_errorcode_t::ERR_FTP_COMMAND_NOT_SUPPORTED,

    /// Server rejected our command because we didn't issue the commands in right
    /// order.
    /// FTP response code 503.
    FtpBadCommandSequence = cef_errorcode_t::ERR_FTP_BAD_COMMAND_SEQUENCE,

    /// PKCS #12 import failed due to incorrect password.
    Pkcs12ImportBadPassword = cef_errorcode_t::ERR_PKCS12_IMPORT_BAD_PASSWORD,

    /// PKCS #12 import failed due to other error.
    Pkcs12ImportFailed = cef_errorcode_t::ERR_PKCS12_IMPORT_FAILED,

    /// CA import failed - not a CA cert.
    ImportCaCertNotCa = cef_errorcode_t::ERR_IMPORT_CA_CERT_NOT_CA,

    /// Import failed - certificate already exists in database.
    /// Note it's a little weird this is an error but reimporting a PKCS12 is ok
    /// (no-op).  That's how Mozilla does it, though.
    ImportCertAlreadyExists = cef_errorcode_t::ERR_IMPORT_CERT_ALREADY_EXISTS,

    /// CA import failed due to some other error.
    ImportCaCertFailed = cef_errorcode_t::ERR_IMPORT_CA_CERT_FAILED,

    /// Server certificate import failed due to some internal error.
    ImportServerCertFailed = cef_errorcode_t::ERR_IMPORT_SERVER_CERT_FAILED,

    /// PKCS #12 import failed due to invalid MAC.
    Pkcs12ImportInvalidMac = cef_errorcode_t::ERR_PKCS12_IMPORT_INVALID_MAC,

    /// PKCS #12 import failed due to invalid/corrupt file.
    Pkcs12ImportInvalidFile = cef_errorcode_t::ERR_PKCS12_IMPORT_INVALID_FILE,

    /// PKCS #12 import failed due to unsupported features.
    Pkcs12ImportUnsupported = cef_errorcode_t::ERR_PKCS12_IMPORT_UNSUPPORTED,

    /// Key generation failed.
    KeyGenerationFailed = cef_errorcode_t::ERR_KEY_GENERATION_FAILED,

    /// Failure to export private key.
    PrivateKeyExportFailed = cef_errorcode_t::ERR_PRIVATE_KEY_EXPORT_FAILED,

    /// Self-signed certificate generation failed.
    SelfSignedCertGenerationFailed = cef_errorcode_t::ERR_SELF_SIGNED_CERT_GENERATION_FAILED,

    /// The certificate database changed in some way.
    CertDatabaseChanged = cef_errorcode_t::ERR_CERT_DATABASE_CHANGED,

    /// DNS resolver received a malformed response.
    DnsMalformedResponse = cef_errorcode_t::ERR_DNS_MALFORMED_RESPONSE,

    /// DNS server requires TCP
    DnsServerRequiresTcp = cef_errorcode_t::ERR_DNS_SERVER_REQUIRES_TCP,

    /// DNS server failed.  This error is returned for all of the following
    /// error conditions:
    /// 1 - Format error - The name server was unable to interpret the query.
    /// 2 - Server failure - The name server was unable to process this query
    ///     due to a problem with the name server.
    /// 4 - Not Implemented - The name server does not support the requested
    ///     kind of query.
    /// 5 - Refused - The name server refuses to perform the specified
    ///     operation for policy reasons.
    DnsServerFailed = cef_errorcode_t::ERR_DNS_SERVER_FAILED,

    /// DNS transaction timed out.
    DnsTimedOut = cef_errorcode_t::ERR_DNS_TIMED_OUT,

    /// The entry was not found in cache, for cache-only lookups.
    DnsCacheMiss = cef_errorcode_t::ERR_DNS_CACHE_MISS,

    /// Suffix search list rules prevent resolution of the given host name.
    DnsSearchEmpty = cef_errorcode_t::ERR_DNS_SEARCH_EMPTY,

    /// Failed to sort addresses according to RFC3484.
    DnsSortError = cef_errorcode_t::ERR_DNS_SORT_ERROR,

    /// Failed to resolve over HTTP, fallback to legacy
    DnsHttpFailed = cef_errorcode_t::ERR_DNS_HTTP_FAILED,
}

/// Implement this trait to handle events related to browser load status. The
/// functions of this trait will be called on the browser process UI thread
/// or render process main thread ([ProcessId::Renderer]).
pub trait LoadHandler<C>: Send + Sync where C: Client {
    /// Called when the loading state has changed. This callback will be executed
    /// twice -- once when loading is initiated either programmatically or by user
    /// action, and once when loading is terminated due to completion, cancellation
    /// of failure. It will be called before any calls to [LoadHandler::on_load_start] and after all
    /// calls to [LoadHandler::on_load_error] and/or [LoadHandler::on_load_end].
    fn on_loading_state_change(
        &self,
        browser: &Browser<C>,
        is_loading: bool,
        can_go_back: bool,
        can_go_forward: bool,
    ) {
    }
    /// Called after a navigation has been committed and before the browser begins
    /// loading contents in the frame. Call
    /// the [Frame::is_main()] function to check if `frame` is the main frame.
    /// `transition_type` provides information about the source of the navigation
    /// and an accurate value is only available in the browser process. Multiple
    /// frames may be loading at the same time. Sub-frames may start or continue
    /// loading after the main frame load has ended. This function will not be
    /// called for same page navigations (fragments, history state, etc.) or for
    /// navigations that fail or are canceled before commit. For notification of
    /// overall browser load status use [LoadHandler::on_loading_state_change] instead.
    fn on_load_start(&self, browser: &Browser<C>, frame: &Frame<C>, transition_type: TransitionType) {}
    /// Called when the browser is done loading a frame. Call the [Frame::is_main()] function to check if `frame` is the
    /// main frame. Multiple frames may be loading at the same time. Sub-frames may
    /// start or continue loading after the main frame load has ended. This
    /// function will not be called for same page navigations (fragments, history
    /// state, etc.) or for navigations that fail or are canceled before commit.
    /// For notification of overall browser load status use [LoadHandler::on_loading_state_change]
    /// instead.
    fn on_load_end(&self, browser: &Browser<C>, frame: &Frame<C>, http_status_code: i32) {}
    /// Called when a navigation fails or is canceled. This function may be called
    /// by itself if before commit or in combination with [LoadHandler::on_load_start]/[LoadHandler::on_load_end] if
    /// after commit. `error_code` is the error code number, `error_text` is the
    /// error text and `failed_url` is the URL that failed to load. See
    /// net\base\net_error_list.h for complete descriptions of the error codes.
    fn on_load_error(
        &self,
        browser: &Browser<C>,
        frame: &Frame<C>,
        error_code: ErrorCode,
        error_text: &str,
        failed_url: &str,
    ) {
    }
}

pub(crate) struct LoadHandlerWrapper<C> where C: Client;

impl<C> LoadHandlerWrapper<C> where C: Client {
    extern "C" fn loading_state_change(
        self_: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        is_loading: std::os::raw::c_int,
        can_go_back: std::os::raw::c_int,
        can_go_forward: std::os::raw::c_int,
    ) {
        let this = unsafe { RefCounted::<cef_load_handler_t>::make_temp(self_) };
        (*this).on_loading_state_change(
            unsafe { &Browser::from_ptr_unchecked(browser) },
            is_loading != 0,
            can_go_back != 0,
            can_go_forward != 0,
        );
    }
    extern "C" fn load_start(
        self_: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        transition_type: cef_transition_type_t,
    ) {
        if let Ok(transition_type) = TransitionType::try_from(transition_type.0) {
            let this = unsafe { RefCounted::<cef_load_handler_t>::make_temp(self_) };
            (*this).on_load_start(
                unsafe { &Browser::from_ptr_unchecked(browser) },
                unsafe { &Frame::from_ptr_unchecked(frame) },
                transition_type,
            );
        }
    }
    extern "C" fn load_end(
        self_: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        http_status_code: std::os::raw::c_int,
    ) {
        let this = unsafe { RefCounted::<cef_load_handler_t>::make_temp(self_) };
        (*this).on_load_end(
            unsafe { &Browser::from_ptr_unchecked(browser) },
            unsafe { &Frame::from_ptr_unchecked(frame) },
            http_status_code,
        );
    }
    extern "C" fn load_error(
        self_: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        error_code: cef_errorcode_t::Type,
        error_text: *const cef_string_t,
        failed_url: *const cef_string_t,
    ) {
        unsafe {
            let this = RefCounted::<cef_load_handler_t>::make_temp(self_);
            (*this).on_load_error(
                &Browser::from_ptr_unchecked(browser),
                &Frame::from_ptr_unchecked(frame),
                ErrorCode::from_unchecked(error_code),
                &CefString::copy_raw_to_string(error_text).unwrap(),
                &CefString::copy_raw_to_string(failed_url).unwrap(),
            );
        }
    }

    pub(crate) fn new(handler: Box<dyn LoadHandler<C>>) -> *mut cef_load_handler_t {
        let rc = RefCounted::new(
            cef_load_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_loading_state_change: Some(Self::loading_state_change),
                on_load_start: Some(Self::load_start),
                on_load_end: Some(Self::load_end),
                on_load_error: Some(Self::load_error),
            },
            handler,
        );
        unsafe { &mut *rc }.get_cef()
    }
}
