const appViewStates = {
    chat: 0,
    members: 1,
}
const userStatuses = {
    offline: 0,
    online: 1,
}

const dataStore = {
    users: null,
    messages: null,
    userId: null,

    init() {
        this.users = new Map()
        this.messages = new Map()
    },
    clear() {
        Alpine.store("system").isClearingData = true

        setTimeout(() => {
            this.users.clear()
            this.messages.clear()

            Alpine.store("system").isClearingData = false
        }, 1e3)
    },
}

const systemStore = {
    isDebug: false,

    isRoomNameLocallyConfirmed: false,
    isUsernameLocallyConfirmed: false,

    error: '',
    isError: false,

    webSocket: null,
    webSocketTimeout: null,
    webSocketPing: null,
    webSocketHeartbeatMessageId: null,
    webSocketLastHeartbeatAt: null,
    webSocketHeartbeatInterval: null,
    isWebSocketConnected: false,

    webRTCPeerConnection: null,
    webRTCDataChannel: null,
    webRTCLastMessageId: 0,
    webRTCTimeout: null,
    webRTCPing: null,
    isWebRTCForceClosing: false,
    webRTCHeartbeatMessageId: null,
    webRTCLastHeartbeatAt: null,
    webRTCHeartbeatInterval: null,
    isWebRTCConnected: false,

    clearDataTimeout: null,
    isClearingData: false,

    sendWebRTCMessage(opCode, payload) {
        if (!this.webRTCPeerConnection || !this.webRTCDataChannel) return

        const message = {i: this.webRTCLastMessageId++, o: opCode}
        if (payload) message.p = payload

        this.webRTCDataChannel.send(msgpackr.pack(message))
    },
    connect(roomName, username) {
        const url = new URL(document.URL)
        let messageId = 0

        const sendWebSocketMessage = (opCode, payload) => {
            if (!this.webSocket) return

            const message = {i: messageId++, o: opCode}
            if (payload) message.p = payload

            this.webSocket.send(msgpackr.pack(message))
        }

        const openWebRTCConnection = () => {
            if (this.webRTCPeerConnection) {
                this.webRTCPeerConnection.close()
                this.webRTCPeerConnection = null
            }
            if (this.webRTCDataChannel) {
                this.webRTCDataChannel.close()
                this.webRTCDataChannel = null
            }

            clearTimeout(this.webRTCTimeout)
            clearInterval(this.webRTCHeartbeatInterval)

            const webRTCReconnect = () => {
                this.isWebRTCConnected = false

                Alpine.store('data').clear()
                sendWebSocketMessage(webSocketOpCodes.request, {
                    t: webSocketPayloadTypes.requestGetRoomSDPOffer,
                    ['room_name']: roomName,
                    username,
                })
            }

            this.webRTCPeerConnection = new RTCPeerConnection({
                iceServers: [{urls: 'stun:stun.l.google.com:19302'}]
            })

            this.webRTCPeerConnection.ondatachannel = event => {
                const reconnect = () => {
                    if (this.isWebRTCForceClosing) return

                    webRTCReconnect()
                    this.isWebRTCForceClosing = false
                }

                this.webRTCDataChannel = event.channel
                this.webRTCDataChannel.onmessage = async message => {
                    const data = msgpackr.unpack(new Uint8Array(message.data instanceof Blob ? await message.data.arrayBuffer() : message.data))
                    const payload = data.p

                    switch (payload?.t) {
                        case webRTCPayloadTypes.dispatchUserUpdate:
                            Alpine.store('data').users.set(payload.user.id, {
                                id: payload.user.id,
                                username: payload.user.username,
                                status: payload.user.status,
                            })

                            break
                        case webRTCPayloadTypes.dispatchMessageUpdate:
                            Alpine.store('data').messages.set(payload.message.id, {
                                id: payload.message.id,
                                authorId: payload.message['author_id'],
                                content: payload.message.content,
                            })

                            break
                        case webRTCPayloadTypes.hello:
                            Alpine.store('data').userId = payload['user_id']

                            for (const user of payload.users) {
                                Alpine.store('data').users.set(user.id, {
                                    id: user.id,
                                    username: user.username,
                                    status: user.status,
                                })
                            }

                            for (const message of payload.messages) {
                                Alpine.store('data').messages.set(message.id, {
                                    id: message.id,
                                    authorId: message['author_id'],
                                    content: message.content,
                                })
                            }
                            break
                        default:
                            if (this.isDebug && data.i === this.webRTCHeartbeatMessageId) {
                                this.webRTCPing = Date.now() - this.webRTCLastHeartbeatAt
                                return
                            }
                    }
                }
                this.webRTCDataChannel.onclose = () => {
                    this.webRTCPing = null
                    this.webRTCLastMessageId = 0
                    this.webRTCHeartbeatMessageId = null
                    this.webRTCLastHeartbeatAt = null
                    this.isWebRTCConnected = false

                    reconnect()
                }
                this.webRTCDataChannel.onerror = () => reconnect()
                this.isWebRTCConnected = true

                this.webRTCHeartbeatInterval = setInterval(() => {
                    this.webRTCHeartbeatMessageId = this.webRTCLastMessageId
                    this.webRTCLastHeartbeatAt = Date.now()

                    this.sendWebRTCMessage(webRTCOpCodes.heartBeat)
                }, this.isDebug ? 1e3 : 5e3)
            }
            // this.webRTCPeerConnection.onicecandidate = event => {
            //     const candidate = event.candidate
            //
            //     if (!candidate) return
            //
            //     sendWebSocketMessage(webSocketOpCodes.request, {
            //         t: webSocketPayloadTypes.requestPostRoomICECandidate,
            //         candidate: candidate.candidate,
            //         'sdp_mid': candidate.sdpMid,
            //         'sdp_m_line_index': candidate.sdpMLineIndex,
            //         'username_fragment': candidate.usernameFragment,
            //     })
            // }
            this.webRTCPeerConnection.onicecandidateerror = error => console.error('ICE Candidate Error', error)
            this.webRTCPeerConnection.onconnectionstatechange = event => {
                clearTimeout(this.webRTCTimeout)

                if (['failed', 'disconnected'].includes(event.target.connectionState)) {
                    webRTCReconnect()
                }
            }
            this.webRTCPeerConnection.onnegotiationneeded = event => console.warn('Negotiation Needed', event)
        }

        const openWebSocketConnection = () => {
            if (this.webSocket) {
                this.webSocket.close()
                this.webSocket = null
            }

            clearTimeout(this.webSocketTimeout)
            clearInterval(this.webSocketHeartbeatInterval)

            this.webSocket = new WebSocket(`${url.protocol === 'https:' ? 'wss' : 'ws'}://${url.host}/ws?encoding=messagePack`)
            this.webSocket.binaryType = 'arraybuffer'
            this.webSocket.onopen = () => {
                this.isWebSocketConnected = true

                sendWebSocketMessage(webSocketOpCodes.authorize, {
                    t: webSocketPayloadTypes.authorize,
                    token: localStorage.getItem('token') || ''
                })
            }
            this.webSocket.onmessage = async message => {
                const data = msgpackr.unpack(new Uint8Array(message.data))
                const payload = data.p

                switch (payload?.t) {
                    case webSocketPayloadTypes.response:
                        const setError = ({error, resetRoomNameConfirmation = false}) => {
                            this.isUsernameLocallyConfirmed = false
                            this.error = error
                            this.isError = true

                            if (resetRoomNameConfirmation) {
                                this.isRoomNameLocallyConfirmed = false

                                if (history.state?.lastHref && !new URL(history.state.lastHref).searchParams.has('room-name')) {
                                    history.back()
                                } else {
                                    const url = new URL(window.location.href)
                                    url.searchParams.delete('room-name')
                                    window.history.replaceState({lastHref: window.location.href}, null, url)
                                }
                            }

                            this.disconnect()
                        }

                        switch (payload.code) {
                            case serverErrors.roomNameTooShort:
                                setError({
                                    error: 'Room name must be at least 3 characters long',
                                    resetRoomNameConfirmation: true,
                                })
                                break
                            case serverErrors.roomNameTooLong:
                                setError({
                                    error: 'Room name must be no more than 3 characters long',
                                    resetRoomNameConfirmation: true,
                                })
                                break
                            case serverErrors.usernameTooShort:
                                setError({
                                    error: 'Username must be at least 3 characters long',
                                })
                                break
                            case serverErrors.usernameTooLong:
                                setError({
                                    error: 'Username must be no more than 3 characters long',
                                })
                                break
                            case serverErrors.usernameTaken:
                                setError({
                                    error: 'This username is already taken in selected room',
                                })
                                break
                        }


                        break
                    case webSocketPayloadTypes.responseSession:
                        localStorage.setItem('token', payload.token)

                        this.webSocketHeartbeatInterval = setInterval(() => {
                            this.webSocketHeartbeatMessageId = messageId
                            this.webSocketLastHeartbeatAt = Date.now()

                            sendWebSocketMessage(webSocketOpCodes.heartBeat)
                        }, this.isDebug ? 1e3 : 30e3)

                        sendWebSocketMessage(webSocketOpCodes.request, {
                            t: webSocketPayloadTypes.requestGetRoomSDPOffer,
                            ['room_name']: roomName,
                            username,
                        })

                        break
                    case webSocketPayloadTypes.responseRoomRtcOffer:
                        openWebRTCConnection()

                        await this.webRTCPeerConnection.setRemoteDescription({
                            type: 'offer',
                            sdp: payload.sdp,
                        })
                        const answer = await this.webRTCPeerConnection.createAnswer()
                        await this.webRTCPeerConnection.setLocalDescription(answer)

                        sendWebSocketMessage(webSocketOpCodes.request, {
                            t: webSocketPayloadTypes.requestPostRoomSDPAnswer,
                            sdp: answer.sdp,
                        })

                        break
                    default:
                        if (this.isDebug && data.i === this.webSocketHeartbeatMessageId) {
                            this.webSocketPing = Date.now() - this.webSocketLastHeartbeatAt
                            return
                        }
                }
            }
            this.webSocket.onclose = event => {
                this.webSocketPing = null
                this.webSocketHeartbeatMessageId = null
                this.webSocketLastHeartbeatAt = null
                this.isWebSocketConnected = false
                this.isWebRTCConnected = false

                clearTimeout(this.webSocketTimeout)
                clearInterval(this.webSocketHeartbeatInterval)

                if (event.code !== 1005) {
                    this.webSocketTimeout = setTimeout(openWebSocketConnection, 1e3)
                }

                if (this.webRTCPeerConnection) {
                    this.webRTCPeerConnection.close()
                    this.webRTCPeerConnection = null
                    this.webRTCDataChannel = null
                }

                Alpine.store('data').clear()
            }
            this.webSocket.onerror = () => {
                this.isWebSocketConnected = false
                this.webSocketTimeout = setTimeout(openWebSocketConnection, 1e3)
            }
        }

        openWebSocketConnection()
    },
    disconnect() {
        if (this.webSocket) {
            this.webSocket.close()
            this.webSocket = null
        }

        if (this.webRTCPeerConnection) {
            this.isWebRTCForceClosing = true

            this.webRTCPeerConnection.close()
            this.webRTCPeerConnection = null
            this.webRTCDataChannel = null
        }

        this.isUsernameLocallyConfirmed = false
        Alpine.store('data').clear()
    },
}

const rootData = {
    // General Information
    enteredRoomName: '',
    enteredUsername: '',
    lastEnteredRoomName: '',
    generalInformationHeadingsHeight: 0,
    connect() {
        this.$store.system.isError = false

        if (!this.$store.system.isRoomNameLocallyConfirmed) {
            this.enteredRoomName = this.enteredRoomName.trim().slice(0, 32)
            if (this.enteredRoomName === '') return

            this.$store.system.isRoomNameLocallyConfirmed = true

            const url = new URL(window.location.href)
            url.searchParams.set('room-name', Base64.encodeURI(this.enteredRoomName))
            window.history.pushState({lastHref: window.location.href}, null, url)

            return
        }

        this.enteredUsername = this.enteredUsername.trim().slice(0, 32)
        if (this.enteredUsername === '') return

        this.$store.system.isUsernameLocallyConfirmed = true

        this.$store.system.connect(this.enteredRoomName, this.enteredUsername)
    },

    // App
    appViewState: appViewStates.chat,
    enteredMessage: '',
    sendMessage() {
        this.enteredMessage = this.enteredMessage.trim()
        if (this.enteredMessage === '') return

        this.$store.system.sendWebRTCMessage(webRTCOpCodes.request, {
            t: webRTCPayloadTypes.requestPostMessage,
            content: this.enteredMessage,
        })

        this.enteredMessage = ''
    },
    get canShowApp() {
        return this.$store.system.isRoomNameLocallyConfirmed
            && this.$store.system.isUsernameLocallyConfirmed
            && this.$store.system.isWebSocketConnected
            && this.$store.system.isWebRTCConnected
    },
    get onlineUsers() {
        let users = []

        this.$store.data.users.forEach(user => {
            if (user.status === userStatuses.online && user.id !== this.$store.data.userId) users.push(user)
        })

        return new Promise(resolve => setTimeout(
            () => resolve(users.sort((a, b) => a.username.localeCompare(b.username))),
            parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--fast-transition-duration').split('s')[0]) * 1000,
        ))
    },
    get offlineUsers() {
        let users = []

        this.$store.data.users.forEach(user => {
            if (user.status === userStatuses.offline && user.id !== this.$store.data.userId) users.push(user)
        })

        return new Promise(resolve => setTimeout(
            () => resolve(users.sort((a, b) => a.username.localeCompare(b.username))),
            parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--fast-transition-duration').split('s')[0]) * 1000,
        ))
    },
    get messagesGroupedByAuthor() {
        let groups = []
        let groupBuffer = []
        let lastAuthorId = null
        let id = 0

        new Map([...this.$store.data.messages].sort((a, b) => {
            if (BigInt(a[0]) > BigInt(b[0])) return 1;
            else if (BigInt(a[0]) === BigInt(b[0])) return 0;
            else if (BigInt(a[0]) < BigInt(b[0])) return -1;
        })).forEach(message => {
            if (lastAuthorId !== null && lastAuthorId !== message.authorId) {
                groups.unshift({
                    id: id++,
                    authorId: lastAuthorId,
                    messages: groupBuffer,
                })

                groupBuffer = []
            }

            if (lastAuthorId !== message.authorId) lastAuthorId = message.authorId

            groupBuffer.push(message)
        })

        if (groupBuffer.length !== 0) {
            groups.unshift({
                id: id++,
                authorId: lastAuthorId,
                messages: groupBuffer,
            })
        }

        return groups
    },

    // Other Methods
    checkParamRoomName(allowHistoryChanging = false) {
        const url = new URL(window.location.href)
        const roomName = url.searchParams.get('room-name')

        if (roomName !== null && roomName.trim() !== '') this.enteredRoomName = Base64.decode(roomName).trim().slice(0, 32)
        else {
            this.enteredRoomName = ''

            if (allowHistoryChanging) {
                url.searchParams.delete('room-name')
                window.history.pushState({lastHref: window.location.href}, null, url)
            }
        }

        this.$store.system.disconnect()

        this.$store.system.isRoomNameLocallyConfirmed = this.enteredRoomName !== ''
        this.$store.system.isUsernameLocallyConfirmed = false
    },
    init() {
        this.$watch('$store.system.isRoomNameLocallyConfirmed', () => {
            if (this.$store.system.isRoomNameLocallyConfirmed) this.lastEnteredRoomName = this.enteredRoomName
        })

        const resizeObserver = new ResizeObserver(entries => {
            if (!this.$store.system.isRoomNameLocallyConfirmed && Array.from(entries[0].target.classList).includes('room-name'))
                this.generalInformationHeadingsHeight = entries[0].target.clientHeight
            else if (
                this.$store.system.isRoomNameLocallyConfirmed
                && !this.$store.system.isUsernameLocallyConfirmed
                && Array.from(entries[0].target.classList).includes('username')
            ) this.generalInformationHeadingsHeight = entries[0].target.clientHeight
        })

        resizeObserver.observe(document.querySelector('#general-information > .headings > h2.room-name'))
        resizeObserver.observe(document.querySelector('#general-information > .headings > h2.username'))

        this.$store.system.isDebug = new URL(document.URL).searchParams.has('debug', 'true')

        window.addEventListener('popstate', () => this.checkParamRoomName())
        this.checkParamRoomName(true)
    },
}

document.addEventListener('alpine:init', () => {
    Alpine.store('data', dataStore)
    Alpine.store('system', systemStore)
    Alpine.data('root', () => (rootData))
})
