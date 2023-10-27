const appViewStates = {
    chat: 0,
    members: 1,
}
const userStatuses = {
    offline: 0,
    online: 1,
}

document.addEventListener('alpine:init', () => {
    Alpine.store('data', {
        users: null,
        messages: null,

        init() {
            this.users = new Map()
            this.messages = new Map()
        },
    })

    Alpine.data('root', () => ({
        // General Information
        roomName: '',
        username: '',
        isRoomNameLocallyConfirmed: false,
        isUsernameLocallyConfirmed: false,
        generalInformationHeadingHeight: 0,
        confirmedRoomName: '',
        temporaryTimeout: null,
        onConnect() {
            if (!this.isRoomNameLocallyConfirmed) {
                this.roomName = this.roomName.trim().slice(0, 32)
                if (this.roomName === '') return

                this.isRoomNameLocallyConfirmed = true

                const url = new URL(window.location.href)
                url.searchParams.set('room-name', Base64.encodeURI(this.roomName))
                window.history.pushState(null, null, url)

                return
            }

            this.username = this.username.trim().slice(0, 32)
            if (this.username !== '') this.isUsernameLocallyConfirmed = true

            clearTimeout(this.temporaryTimeout)
            this.temporaryTimeout = setTimeout(() => {
                this.isWebSocketConnected = true
                this.isWebRTCConnected = true

                this.me = '6'

                // Fake users
                this.$store.data.users.set('0', {
                    id: '0',
                    username: 'John',
                    status: userStatuses.online,
                })
                this.$store.data.users.set('1', {
                    id: '1',
                    username: 'Flappy Brow',
                    status: userStatuses.offline,
                })
                this.$store.data.users.set('2', {
                    id: '2',
                    username: 'Vouter',
                    status: userStatuses.offline,
                })
                this.$store.data.users.set('3', {
                    id: '3',
                    username: 'Mr. Admin',
                    status: userStatuses.online,
                })
                this.$store.data.users.set('4', {
                    id: '4',
                    username: 'Viktoriya',
                    status: userStatuses.online,
                })
                this.$store.data.users.set('5', {
                    id: '5',
                    username: 'Guest',
                    status: userStatuses.online,
                })
                this.$store.data.users.set('6', {
                    id: '6',
                    username: this.username,
                    status: userStatuses.online,
                })

                // Fake messages
                this.$store.data.messages.set('100', {
                    id: '100',
                    authorId: '0',
                    content: 'Hey everyone! I just came across this fascinating article about the future of artificial intelligence. It\'s mind-blowing how quickly technology is advancing. 😲',
                })
                this.$store.data.messages.set('101', {
                    id: '101',
                    authorId: '0',
                    content: 'I\'m curious to hear your thoughts on the ethical implications of AI. Do you think we\'re doing enough to ensure responsible development?',
                })
                this.$store.data.messages.set('102', {
                    id: '102',
                    authorId: '0',
                    content: 'By the way, I\'ve been experimenting with a new machine learning algorithm for natural language processing. It\'s been a real challenge, but also a lot of fun. Anyone else into AI development here?',
                })
                this.$store.data.messages.set('103', {
                    id: '103',
                    authorId: '4',
                    content: 'Thanks for sharing that article, John! I read it earlier today, and it\'s mind-blowing indeed. The ethical concerns around AI are a hot topic right now. I think we\'re making progress, but there\'s still a long way to go in ensuring responsible AI development.',
                })
                this.$store.data.messages.set('104', {
                    id: '104',
                    authorId: '3',
                    content: 'Hey, I\'m not as deep into AI as you guys, but I find the topic really interesting. The ethical discussions are crucial, and I think it\'s vital that we prioritize responsible AI.',
                })
                this.$store.data.messages.set('105', {
                    id: '105',
                    authorId: '3',
                    content: 'Viktoriya, your chatbot project sounds intriguing. Can you tell me more about it?',
                })
                this.$store.data.messages.set('106', {
                    id: '106',
                    authorId: '0',
                    content: 'That sounds awesome, Viktoriya! I\'d love to collaborate on the chatbot project. I\'ll definitely check out your GitHub repo.',
                })
                this.$store.data.messages.set('107', {
                    id: '107',
                    authorId: '0',
                    content: 'Also, for anyone interested, I recommend checking out this AI podcast I recently discovered. They discuss some really thought-provoking topics related to AI ethics and development. 🎙️🤖',
                })
            }, 400)
        },

        // App
        isWebSocketConnected: false,
        isWebRTCConnected: false,
        appViewState: appViewStates.chat,
        me: null,
        enteredMessage: '',
        sendMessage() {
            this.enteredMessage = this.enteredMessage.trim()
            if (this.enteredMessage === '') return

            const id = Date.now().toString()
            const content = this.enteredMessage
            this.$store.data.messages.set(id, {
                id: id,
                authorId: this.me,
                content: content,
                isSending: true,
            })

            // Fake delivery
            setTimeout(() => {
                this.$store.data.messages.set(id, {
                    id: id,
                    authorId: this.me,
                    content: content,
                })
            }, Math.floor(Math.random() * 1e3))

            this.enteredMessage = ''
        },
        get canShowApp() {
            return this.isRoomNameLocallyConfirmed
                && this.isUsernameLocallyConfirmed
                && this.isWebSocketConnected
                && this.isWebRTCConnected
        },
        get onlineUsers() {
            const fastTransitionDuration = parseFloat(
                getComputedStyle(document.documentElement)
                    .getPropertyValue('--fast-transition-duration')
                    .split('s')[0]
            ) * 1000
            let users = []

            this.$store.data.users.forEach(user => {
                if (user.status === userStatuses.online && user.id !== this.me) users.push(user)
            })

            return new Promise(resolve => setTimeout(
                () => resolve(users.sort((a, b) => a.username.localeCompare(b.username))),
                fastTransitionDuration,
            ))
        },
        get offlineUsers() {
            const fastTransitionDuration = parseFloat(
                getComputedStyle(document.documentElement)
                    .getPropertyValue('--fast-transition-duration')
                    .split('s')[0]
            ) * 1000
            let users = []

            this.$store.data.users.forEach(user => {
                if (user.status === userStatuses.offline && user.id !== this.me) users.push(user)
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

            groups.unshift({
                id: id++,
                authorId: lastAuthorId,
                messages: groupBuffer,
            })

            return groups
        },

        // Other Methods
        checkParamRoomName(allowHistoryChanging = false) {
            const url = new URL(window.location.href)
            const roomName = url.searchParams.get('room-name')

            if (roomName !== null && roomName.trim() !== '') this.roomName = Base64.decode(roomName).trim().slice(0, 32)
            else {
                this.roomName = ''

                if (allowHistoryChanging) {
                    url.searchParams.delete('room-name')
                    window.history.pushState(null, null, url)
                }
            }

            this.isRoomNameLocallyConfirmed = this.roomName !== ''
            this.isUsernameLocallyConfirmed = false
        },
        init() {
            this.$watch('isRoomNameLocallyConfirmed', () => {
                if (this.isRoomNameLocallyConfirmed) this.confirmedRoomName = this.roomName
            })

            const resizeObserver = new ResizeObserver(entries => {
                if (!this.isRoomNameLocallyConfirmed && Array.from(entries[0].target.classList).includes('room-name'))
                    this.generalInformationHeadingHeight = entries[0].target.clientHeight
                else if (
                    this.isRoomNameLocallyConfirmed
                    && !this.isUsernameLocallyConfirmed
                    && Array.from(entries[0].target.classList).includes('username')
                ) this.generalInformationHeadingHeight = entries[0].target.clientHeight
            })

            resizeObserver.observe(document.querySelector('#general-information > .headings > h2.room-name'))
            resizeObserver.observe(document.querySelector('#general-information > .headings > h2.username'))

            window.addEventListener('popstate', () => this.checkParamRoomName())

            this.checkParamRoomName(true)
        },
    }))
})

// Boring Avatars
const avatarSize = 36

const hashCode = (name) => {
    let hash = 0

    for (let i = 0; i < name.length; i++) {
        const character = name.charCodeAt(i)

        hash = ((hash << 5) - hash) + character
        hash = hash & hash
    }

    return Math.abs(hash)
}

const getDigit = (number, ntn) => {
    return Math.floor((number / Math.pow(10, ntn)) % 10)
}

const getBoolean = (number, ntn) => {
    return (!((getDigit(number, ntn)) % 2))
}

const getUnit = (number, range, index) => {
    let value = number % range

    if (index && ((getDigit(number, index) % 2) === 0)) return -value
    else return value
}

const getRandomColor = (number, colors, range) => {
    return colors[(number) % range]
}

const getContrast = (hexColor) => {
    if (hexColor.slice(0, 1) === '#') hexColor = hexColor.slice(1)

    const r = parseInt(hexColor.slice(0, 2), 16)
    const g = parseInt(hexColor.slice(2, 2 + 2), 16)
    const b = parseInt(hexColor.slice(4, 4 + 2), 16)
    const yiq = ((r * 299) + (g * 587) + (b * 114)) / 1000

    return (yiq >= 128) ? '#000000' : '#ffffff'

}

const generateAvatar = (username) => {
    const props = {
        colors: ['#ef4335', '#f68b36', '#f2cd4f', '#cae081', '#88eed0'],
        username: username || '',
        size: 40,
    }

    const numFromName = hashCode(props.username)
    const range = props.colors && props.colors.length
    const wrapperColor = getRandomColor(numFromName, props.colors, range)
    const preTranslateX = getUnit(numFromName, 10, 1)
    const wrapperTranslateX = preTranslateX < 5 ? preTranslateX + avatarSize / 9 : preTranslateX
    const preTranslateY = getUnit(numFromName, 10, 2)
    const wrapperTranslateY = preTranslateY < 5 ? preTranslateY + avatarSize / 9 : preTranslateY

    const data = {
        wrapperColor: wrapperColor,
        faceColor: getContrast(wrapperColor),
        backgroundColor: getRandomColor(numFromName + 13, props.colors, range),
        wrapperTranslateX: wrapperTranslateX,
        wrapperTranslateY: wrapperTranslateY,
        wrapperRotate: getUnit(numFromName, 360),
        wrapperScale: 1 + getUnit(numFromName, avatarSize / 12) / 10,
        isMouthOpen: getBoolean(numFromName, 2),
        isCircle: getBoolean(numFromName, 1),
        eyeSpread: getUnit(numFromName, 5),
        mouthSpread: getUnit(numFromName, 3),
        faceRotate: getUnit(numFromName, 10, 3),
        faceTranslateX:
            wrapperTranslateX > avatarSize / 6 ? wrapperTranslateX / 2 : getUnit(numFromName, 8, 1),
        faceTranslateY:
            wrapperTranslateY > avatarSize / 6 ? wrapperTranslateY / 2 : getUnit(numFromName, 7, 2),
    }

    const mouth = data.isMouthOpen ? `
        <path
            d="M15 ${19 + data.mouthSpread}c2 1 4 1 6 0"
            stroke="${data.faceColor}"
            fill="none"
            stroke-linecap="round"
        />` : `
        <path
            d="M13,${19 + data.mouthSpread} a1,0.75 0 0,0 10,0"
            fill="${data.faceColor}"
        />`

    return `
        <svg
            viewBox="0 0 ${avatarSize} ${avatarSize}"
            fill="none"
            role="img"
            xmlns="http://www.w3.org/2000/svg"
            width="${props.size}"
            height="${props.size}"
        >
            <mask id="mask" maskUnits="userSpaceOnUse" x="0" y="0" width="${avatarSize}" height="${avatarSize}">
                <rect width="${avatarSize}" height="${avatarSize}" fill="#ffffff" />
            </mask>
            <g mask="url(mask)">
                <rect width="${avatarSize}" height="${avatarSize}" fill="${data.backgroundColor}" />
                <rect
                    x="0"
                    y="0"
                    width="${avatarSize}"
                    height="${avatarSize}"
                    transform="translate(${data.wrapperTranslateX} ${data.wrapperTranslateY}) rotate(${data.wrapperRotate} ${avatarSize / 2} ${avatarSize / 2}) scale(${data.wrapperScale})"
                    fill="${data.wrapperColor}"
                    rx="${data.isCircle ? avatarSize : avatarSize / 6}"
                />
                <g transform="translate(${data.faceTranslateX} ${data.faceTranslateY}) rotate(${data.faceRotate} ${avatarSize / 2} ${avatarSize / 2})">
                    ${mouth}
                    <rect
                        x="${14 - data.eyeSpread}"
                        y="14"
                        width="1.5"
                        height="2"
                        rx="1"
                        stroke="none"
                        fill="${data.faceColor}"
                    />
                    <rect
                        x="${20 + data.eyeSpread}"
                        y="14"
                        width="1.5"
                        height="2"
                        rx="1"
                        stroke="none"
                        fill="${data.faceColor}"
                    />
                </g>
            </g>
        </svg>
    `
}
