import QtQuick
import qs.Common
import qs.Widgets

Item {
    id: slider

    property int value: 50
    property int minimum: 0
    property int maximum: 100
    property int step: 1
    property string topIcon: ""
    property string bottomIcon: ""
    property string unit: "%"
    property bool showValue: true
    property bool isDragging: false
    property bool wheelEnabled: true
    property bool centerMinimum: false
    property real valueOverride: -1
    property bool alwaysShowValue: false
    readonly property bool containsMouse: sliderMouseArea.containsMouse

    property color thumbOutlineColor: Theme.surfaceContainer
    property color trackColor: enabled ? Theme.outline : Theme.outline
    property real trackOpacity: Theme.popupTransparency

    signal sliderValueChanged(int newValue)
    signal sliderDragFinished(int finalValue)

    width: 48
    implicitHeight: 300

    function updateValueFromPosition(y) {
        let ratio = 1 - Math.max(0, Math.min(1, (y - sliderHandle.height / 2) / (sliderTrack.height - sliderHandle.height)));

        if (centerMinimum) {
            ratio = Math.max(0, (ratio - 0.5) * 2);
        }

        let rawValue = minimum + ratio * (maximum - minimum);
        let newValue = step > 1 ? Math.round(rawValue / step) * step : Math.round(rawValue);

        newValue = Math.max(minimum, Math.min(maximum, newValue));
        if (newValue !== value) {
            value = newValue;
            sliderValueChanged(newValue);
        }
    }

    Column {
        anchors.fill: parent
        width: parent.width
        spacing: Theme.spacingM

        DankIcon {
            name: slider.topIcon
            size: Theme.iconSize
            color: slider.enabled ? Theme.surfaceText : Theme.onSurface_38
            anchors.horizontalCenter: parent.horizontalCenter
            visible: slider.topIcon.length > 0
        }

        StyledRect {
            id: sliderTrack

            property int topIconHeight: slider.topIcon.length > 0 ? Theme.iconSize : 0
            property int bottomIconHeight: slider.bottomIcon.length > 0 ? Theme.iconSize : 0

            height: parent.height - (topIconHeight + bottomIconHeight + (slider.topIcon.length > 0 ? Theme.spacingM : 0) + (slider.bottomIcon.length > 0 ? Theme.spacingM : 0))

            width: 12
            radius: Theme.cornerRadius
            color: Theme.withAlpha(slider.trackColor, slider.trackOpacity)
            anchors.horizontalCenter: parent.horizontalCenter
            clip: false

            StyledRect {
                id: sliderFill
                width: parent.width
                radius: Theme.cornerRadius
                topRightRadius: 0
                topLeftRadius: 0
                bottomLeftRadius: Theme.cornerRadius
                bottomRightRadius: Theme.cornerRadius
                height: {
                    const range = slider.maximum - slider.minimum;
                    const rawRatio = range === 0 ? 0 : (slider.value - slider.minimum) / range;
                    const ratio = slider.centerMinimum ? (0.5 + rawRatio * 0.5) : rawRatio;
                    const travel = sliderTrack.height - sliderHandle.height;
                    const handleBottom = travel * ratio;
                    const endPoint = handleBottom + (sliderHandle.height / 2);
                    return Math.max(0, Math.min(sliderTrack.height, endPoint));
                }
                anchors.bottom: parent.bottom
                color: slider.enabled ? Theme.primary : Theme.withAlpha(Theme.onSurface, 0.12)
            }

            StyledRect {
                id: sliderHandle

                property bool active: sliderMouseArea.containsMouse || sliderMouseArea.pressed || slider.isDragging

                width: 20
                height: 4
                radius: Theme.cornerRadius
                y: {
                    const range = slider.maximum - slider.minimum;
                    const rawRatio = range === 0 ? 0 : (slider.value - slider.minimum) / range;
                    const ratio = slider.centerMinimum ? (0.5 + rawRatio * 0.5) : rawRatio;
                    const travel = sliderTrack.height - height;
                    return Math.max(0, Math.min(travel, travel * (1 - ratio)));
                }
                anchors.horizontalCenter: parent.horizontalCenter
                color: slider.enabled ? Theme.primary : Theme.withAlpha(Theme.onSurface, 0.12)
                border.width: 0
                border.color: slider.thumbOutlineColor

                StyledRect {
                    anchors.fill: parent
                    radius: Theme.cornerRadius
                    color: Theme.onPrimary
                    opacity: slider.enabled ? (sliderMouseArea.pressed ? 0.16 : (sliderMouseArea.containsMouse ? 0.08 : 0)) : 0
                    visible: opacity > 0
                }

                StyledRect {
                    anchors.centerIn: parent
                    width: parent.width + 20
                    height: parent.height + 20
                    radius: width / 2
                    color: "transparent"
                    border.width: 2
                    border.color: Theme.primary
                    opacity: slider.enabled && slider.focus ? 0.3 : 0
                    visible: opacity > 0
                }

                Rectangle {
                    id: ripple
                    anchors.centerIn: parent
                    width: 0
                    height: 0
                    radius: width / 2
                    color: Theme.onPrimary
                    opacity: 0

                    function start() {
                        opacity = 0.16;
                        width = 0;
                        height = 0;
                        rippleAnimation.start();
                    }

                    SequentialAnimation {
                        id: rippleAnimation
                        NumberAnimation {
                            target: ripple
                            properties: "width,height"
                            to: 28
                            duration: 180
                        }
                        NumberAnimation {
                            target: ripple
                            property: "opacity"
                            to: 0
                            duration: 150
                        }
                    }
                }

                TapHandler {
                    acceptedButtons: Qt.LeftButton
                    onPressedChanged: {
                        if (pressed && slider.enabled) {
                            ripple.start();
                        }
                    }
                }

                scale: active ? 1.05 : 1.0

                Behavior on scale {
                    NumberAnimation {
                        duration: Theme.shortDuration
                        easing.type: Theme.standardEasing
                    }
                }
            }

            Item {
                id: sliderContainer

                anchors.fill: parent

                MouseArea {
                    id: sliderMouseArea

                    property bool isDragging: false

                    anchors.fill: parent
                    anchors.leftMargin: -10
                    anchors.rightMargin: -10
                    hoverEnabled: true
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    enabled: slider.enabled
                    preventStealing: true
                    acceptedButtons: Qt.LeftButton
                    onWheel: wheelEvent => {
                        if (!slider.wheelEnabled) {
                            wheelEvent.accepted = false;
                            return;
                        }
                        let wheelStep = slider.step > 1 ? slider.step : Math.max(1, (maximum - minimum) / 100);
                        let newValue = wheelEvent.angleDelta.y > 0 ? Math.min(maximum, value + wheelStep) : Math.max(minimum, value - wheelStep);
                        if (slider.step > 1)
                            newValue = Math.round(newValue / slider.step) * slider.step;
                        newValue = Math.round(newValue);
                        if (newValue !== value) {
                            value = newValue;
                            sliderValueChanged(newValue);
                        }
                        wheelEvent.accepted = true;
                    }
                    onPressed: mouse => {
                        if (slider.enabled) {
                            slider.isDragging = true;
                            sliderMouseArea.isDragging = true;
                            updateValueFromPosition(mouse.y);
                        }
                    }
                    onReleased: {
                        if (slider.enabled) {
                            slider.isDragging = false;
                            sliderMouseArea.isDragging = false;
                            slider.sliderDragFinished(slider.value);
                        }
                    }
                    onPositionChanged: mouse => {
                        if (pressed && slider.isDragging && slider.enabled) {
                            updateValueFromPosition(mouse.y);
                        }
                    }
                    onClicked: mouse => {
                        if (slider.enabled && !slider.isDragging) {
                            updateValueFromPosition(mouse.y);
                        }
                    }
                }
            }

            StyledRect {
                id: valueTooltip

                width: tooltipText.reservedWidth + Theme.spacingS * 2
                height: tooltipText.contentHeight + Theme.spacingXS * 2
                radius: Theme.cornerRadius
                color: Theme.surfaceContainer
                border.color: Theme.outline
                border.width: 1
                anchors.left: parent.right
                anchors.leftMargin: Theme.spacingM
                y: Math.max(0, Math.min(parent.height - height, sliderHandle.y + sliderHandle.height / 2 - height / 2))
                visible: slider.alwaysShowValue ? slider.showValue : ((sliderMouseArea.containsMouse && slider.showValue) || (slider.isDragging && slider.showValue))
                opacity: visible ? 1 : 0

                NumericText {
                    id: tooltipText

                    text: (slider.valueOverride >= 0 ? Math.round(slider.valueOverride) : slider.value) + slider.unit
                    reserveText: {
                        let widest = "";
                        const samples = [slider.minimum, slider.maximum];
                        if (slider.valueOverride >= 0)
                            samples.push(slider.valueOverride);
                        for (let i = 0; i < samples.length; i++) {
                            const candidate = Math.round(samples[i]) + slider.unit;
                            if (candidate.length > widest.length)
                                widest = candidate;
                        }
                        return widest;
                    }
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.surfaceText
                    font.weight: Font.Medium
                    anchors.centerIn: parent
                    font.hintingPreference: Font.PreferFullHinting
                }

                Behavior on opacity {
                    NumberAnimation {
                        duration: Theme.shortDuration
                        easing.type: Theme.standardEasing
                    }
                }
            }
        }

        DankIcon {
            name: slider.bottomIcon
            size: Theme.iconSize
            color: slider.enabled ? Theme.surfaceText : Theme.onSurface_38
            anchors.horizontalCenter: parent.horizontalCenter
            visible: slider.bottomIcon.length > 0
        }
    }
}
