const webSocketOpCodes = {
    heartBeat: 0,
    request: 1,
    response: 2,
    error: 3,
    authorize: 4,
}
const webSocketPayloadTypes = {
    // Request
    requestGetRoomRtcOffer: 10,
    requestPostRoomRtcAnswer: 11,

    // Response
    response: 20,
    responseSession: 21,
    responseRoomRtcOffer: 22,

    // Authorize
    authorize: 30,

    // Other
    null: 0,
}
const webSocketCloseErrors = {
    unknown: 4000,
    opcode: 4001,
    invalidMessage: 4002,
    notAuthenticated: 4003,
    authenticationFailed: 4004,
    alreadyAuthenticated: 4005,
}
const serverErrors = {
    // Default HTTP errors
    badRequest: 400,
    unauthorized: 401,
    forbidden: 403,
    notFound: 404,
    methodNotAllowed: 405,
    conflict: 409,
    internalError: 500,

    // Minimum / Maximum number of ... reached
    roomNameTooShort: 3001,
    roomNameTooLong: 3002,
    usernameTooShort: 3003,
    usernameTooLong: 3004,

    // Invalid body or something else
    usernameTaken: 4001,
}

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
