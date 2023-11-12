const fastTransitionDuration = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--fast-transition-duration').split('s')[0]) * 1000

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
    insertFakeData(enteredUsername) {
        this.init()

        Alpine.store("system").isClearingData = false

        this.userId = '6'

        // Fake users
        this.users.set('0', {
            id: '0',
            username: 'John',
            status: userStatuses.online,
        })
        this.users.set('1', {
            id: '1',
            username: 'Flappy Brow',
            status: userStatuses.offline,
        })
        this.users.set('2', {
            id: '2',
            username: 'Vouter',
            status: userStatuses.offline,
        })
        this.users.set('3', {
            id: '3',
            username: 'Mr. Admin',
            status: userStatuses.online,
        })
        this.users.set('4', {
            id: '4',
            username: 'Viktoriya',
            status: userStatuses.online,
        })
        this.users.set('5', {
            id: '5',
            username: 'Guest',
            status: userStatuses.online,
        })
        this.users.set('6', {
            id: '6',
            username: enteredUsername,
            status: userStatuses.online,
        })

        // Fake messages
        this.messages.set('100', {
            id: '100',
            authorId: '0',
            content: 'Hey everyone! I just came across this fascinating article about the future of artificial intelligence. It\'s mind-blowing how quickly technology is advancing. 😲',
        })
        this.messages.set('101', {
            id: '101',
            authorId: '0',
            content: 'I\'m curious to hear your thoughts on the ethical implications of AI. Do you think we\'re doing enough to ensure responsible development?',
        })
        this.messages.set('102', {
            id: '102',
            authorId: '0',
            content: 'By the way, I\'ve been experimenting with a new machine learning algorithm for natural language processing. It\'s been a real challenge, but also a lot of fun. Anyone else into AI development here?',
        })
        this.messages.set('103', {
            id: '103',
            authorId: '4',
            content: 'Thanks for sharing that article, John! I read it earlier today, and it\'s mind-blowing indeed. The ethical concerns around AI are a hot topic right now. I think we\'re making progress, but there\'s still a long way to go in ensuring responsible AI development.',
        })
        this.messages.set('104', {
            id: '104',
            authorId: '3',
            content: 'Hey, I\'m not as deep into AI as you guys, but I find the topic really interesting. The ethical discussions are crucial, and I think it\'s vital that we prioritize responsible AI.',
        })
        this.messages.set('105', {
            id: '105',
            authorId: '3',
            content: 'Viktoriya, your chatbot project sounds intriguing. Can you tell me more about it?',
        })
        this.messages.set('106', {
            id: '106',
            authorId: '0',
            content: 'That sounds awesome, Viktoriya! I\'d love to collaborate on the chatbot project. I\'ll definitely check out your GitHub repo.',
        })
        this.messages.set('107', {
            id: '107',
            authorId: '0',
            content: 'Also, for anyone interested, I recommend checking out this AI podcast I recently discovered. They discuss some really thought-provoking topics related to AI ethics and development. 🎙️🤖',
        })
    },
    clear() {
        Alpine.store("system").isClearingData = true
    },
}

const systemStore = {
    isRoomNameLocallyConfirmed: false,
    isUsernameLocallyConfirmed: false,

    error: '',
    isError: false,

    webSocket: null,
    webSocketTimeout: null,
    webSocketHeartbeatInterval: null,
    isWebSocketConnected: false,

    isWebRTCConnected: false,

    clearDataTimeout: null,
    isClearingData: false,

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
            this.isWebRTCConnected = true
            Alpine.store('data').insertFakeData(username)
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
            this.webSocket.onmessage = message => {
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
                            sendWebSocketMessage(webSocketOpCodes.heartBeat)
                        }, 30e3)

                        sendWebSocketMessage(webSocketOpCodes.request, {
                            t: webSocketPayloadTypes.requestGetRoomRtcOffer,
                            ['room_name']: roomName,
                            username,
                        })

                        break
                    case webSocketPayloadTypes.responseRoomRtcOffer:
                        openWebRTCConnection()

                        sendWebSocketMessage(webSocketOpCodes.request, {
                            t: webSocketPayloadTypes.requestGetRoomRtcOffer,
                            // TODO: Send real SDP
                            sdp: '',
                        })

                        break
                }
            }
            this.webSocket.onclose = event => {
                this.isWebSocketConnected = false
                this.isWebRTCConnected = false

                if (event.code !== 1005) {
                    this.webSocketTimeout = setTimeout(() => openWebSocketConnection(), 1e3)
                } else {
                    this.isUsernameLocallyConfirmed = false
                }

                Alpine.store('data').clear()
            }
            this.webSocket.onerror = () => {
                this.isWebSocketConnected = false
                this.webSocketTimeout = setTimeout(() => openWebSocketConnection(), 1e3)
            }
        }

        openWebSocketConnection()
    },
    disconnect() {
        if (this.webSocket) {
            this.webSocket.close()
            this.webSocket = null
        }
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

        const id = Date.now().toString()
        const content = this.enteredMessage
        this.$store.data.messages.set(id, {
            id: id,
            authorId: this.$store.data.userId,
            content: content,
            isSending: true,
        })

        // Fake delivery
        setTimeout(() => {
            this.$store.data.messages.set(id, {
                id: id,
                authorId: this.$store.data.userId,
                content: content,
            })
        }, Math.floor(Math.random() * 1e3))

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
            fastTransitionDuration,
        ))
    },
    get offlineUsers() {
        let users = []

        this.$store.data.users.forEach(user => {
            if (user.status === userStatuses.offline && user.id !== this.$store.data.userId) users.push(user)
        })

        return new Promise(resolve => setTimeout(
            () => resolve(users.sort((a, b) => a.username.localeCompare(b.username))),
            fastTransitionDuration,
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

        window.addEventListener('popstate', () => this.checkParamRoomName())

        this.checkParamRoomName(true)
    },
}

document.addEventListener('alpine:init', () => {
    Alpine.store('data', dataStore)
    Alpine.store('system', systemStore)
    Alpine.data('root', () => (rootData))
})
