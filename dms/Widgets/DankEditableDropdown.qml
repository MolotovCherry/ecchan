pragma ComponentBehavior: Bound

import "fzf.js" as Fzf
import QtQuick
import QtQuick.Controls
import Quickshell
import qs.Common
import qs.Widgets

Item {
    id: root

    LayoutMirroring.enabled: I18n.isRtl
    LayoutMirroring.childrenInherit: true

    function checkParentDisablesTransparency() {
        let p = parent;
        while (p) {
            if (p.disablePopupTransparency === true)
                return true;
            p = p.parent;
        }
        return false;
    }

    property bool isEditing: false
    property string addNewTextEntry: ""
    property string addNewTextEditPlaceholder: ""

    property string text: ""
    property string description: ""
    property string currentValue: currentIdx >= 0 && currentIdx < options.length ? options[currentIdx] : ""
    property int currentIdx: -1
    property var options: []
    property var optionIcons: []
    property bool enableFuzzySearch: false
    property var optionIconMap: ({})

    function rebuildIconMap() {
        const map = {};
        for (let i = 0; i < options.length; i++) {
            if (optionIcons.length > i)
                map[options[i]] = optionIcons[i];
        }
        optionIconMap = map;
    }

    onOptionsChanged: rebuildIconMap()
    onOptionIconsChanged: rebuildIconMap()

    property int popupWidthOffset: 0
    property int maxPopupHeight: 400
    property bool openUpwards: false
    property int popupWidth: 0
    property bool alignPopupRight: false
    property int dropdownWidth: 200
    property bool compactMode: text === "" && description === ""
    property bool addHorizontalPadding: false
    property string emptyText: ""
    property bool usePopupTransparency: !checkParentDisablesTransparency()

    signal valueChanged(int idx, string value)
    signal valueDeleted(int idx, string value)
    signal valueAdded(int idx, string value)

    function closeDropdownMenu() {
        dropdownMenu.close();
    }

    function resetSearch() {
        searchField.text = "";
        dropdownMenu.fzfFinder = null;
        dropdownMenu.searchQuery = "";
        dropdownMenu.selectedIndex = -1;
    }

    width: compactMode ? dropdownWidth : parent.width
    implicitHeight: compactMode ? 40 : Math.max(60, labelColumn.implicitHeight + Theme.spacingM)

    Component.onDestruction: {
        if (dropdownMenu.visible)
            dropdownMenu.close();
    }

    Column {
        id: labelColumn

        anchors.left: parent.left
        anchors.right: dropdown.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: root.addHorizontalPadding ? Theme.spacingM : 0
        anchors.rightMargin: Theme.spacingL
        spacing: Theme.spacingXS
        visible: !root.compactMode

        StyledText {
            text: root.text
            font.pixelSize: Theme.fontSizeMedium
            color: Theme.surfaceText
            font.weight: Font.Medium
            width: parent.width
            horizontalAlignment: Text.AlignLeft
        }

        StyledText {
            text: root.description
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.surfaceVariantText
            visible: root.description.length > 0
            wrapMode: Text.WordWrap
            width: parent.width
            horizontalAlignment: Text.AlignLeft
        }
    }

    Rectangle {
        id: dropdown

        width: root.compactMode ? parent.width : (root.popupWidth === -1 ? undefined : (root.popupWidth > 0 ? root.popupWidth : root.dropdownWidth))
        height: 40
        anchors.right: parent.right
        anchors.rightMargin: root.addHorizontalPadding && !root.compactMode ? Theme.spacingM : 0
        anchors.verticalCenter: parent.verticalCenter
        radius: Theme.cornerRadius
        color: dropdownArea.containsMouse || dropdownMenu.visible ? Theme.surfaceContainerHigh : (root.usePopupTransparency ? Theme.withAlpha(Theme.surfaceContainer, Theme.popupTransparency) : Theme.surfaceContainer)
        border.color: dropdownMenu.visible ? Theme.primary : Qt.rgba(Theme.outline.r, Theme.outline.g, Theme.outline.b, 0.2)
        border.width: dropdownMenu.visible ? 2 : 1

        MouseArea {
            id: dropdownArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                if (dropdownMenu.visible) {
                    dropdownMenu.close();
                    return;
                }

                dropdownMenu.open();

                let currentIndex = root.currentIdx;
                listView.positionViewAtIndex(currentIndex, ListView.Beginning);

                const pos = dropdown.mapToItem(Overlay.overlay, 0, 0);
                const popupW = dropdownMenu.width;
                const popupH = dropdownMenu.height;
                const overlayH = Overlay.overlay.height;
                const goUp = root.openUpwards || pos.y + dropdown.height + popupH + 4 > overlayH;
                dropdownMenu.x = root.alignPopupRight ? pos.x + dropdown.width - popupW : pos.x - (root.popupWidthOffset / 2);
                dropdownMenu.y = goUp ? pos.y - popupH - 4 : pos.y + dropdown.height + 4;
                if (root.enableFuzzySearch) {
                    searchField.forceActiveFocus();
                }
            }
        }

        Row {
            id: contentRow

            anchors.left: parent.left
            anchors.right: expandIcon.left
            anchors.verticalCenter: parent.verticalCenter
            anchors.leftMargin: Theme.spacingM
            anchors.rightMargin: Theme.spacingS
            spacing: Theme.spacingS

            DankIcon {
                name: root.optionIconMap[root.currentValue] ?? ""
                size: 18
                color: Theme.surfaceText
                anchors.verticalCenter: parent.verticalCenter
                visible: name !== ""
            }

            StyledText {
                text: root.currentValue
                font.pixelSize: Theme.fontSizeMedium
                color: Theme.surfaceText
                anchors.verticalCenter: parent.verticalCenter
                width: contentRow.width - (contentRow.children[0].visible ? contentRow.children[0].width + contentRow.spacing : 0)
                elide: Text.ElideRight
                wrapMode: Text.NoWrap
                horizontalAlignment: Text.AlignLeft
            }
        }

        DankIcon {
            id: expandIcon

            name: dropdownMenu.visible ? "expand_less" : "expand_more"
            size: 20
            color: Theme.surfaceText
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.rightMargin: Theme.spacingS

            Behavior on rotation {
                NumberAnimation {
                    duration: Theme.shortDuration
                    easing.type: Theme.standardEasing
                }
            }
        }
    }

    Popup {
        id: dropdownMenu

        property string searchQuery: ""
        property var filteredOptions: {
            if (!root.enableFuzzySearch || searchQuery.length === 0) {
                return [...root.options, root.addNewTextEntry];
            }
            if (!fzfFinder) {
                return [...root.options, root.addNewTextEntry];
            }

            return fzfFinder.find(searchQuery).map(r => r.item);
        }
        property int selectedIndex: -1
        property var fzfFinder: null

        function initFinder() {
            fzfFinder = new Fzf.Finder(root.options, {
                "selector": option => option,
                "limit": 50,
                "casing": "case-insensitive",
                "sort": true,
                "tiebreakers": [(a, b, selector) => selector(a.item).length - selector(b.item).length]
            });
        }

        function selectNext() {
            if (filteredOptions.length === 0)
                return;
            selectedIndex = (selectedIndex + 1) % filteredOptions.length;
            listView.positionViewAtIndex(selectedIndex, ListView.Contain);
        }

        function selectPrevious() {
            if (filteredOptions.length === 0)
                return;
            selectedIndex = selectedIndex <= 0 ? filteredOptions.length - 1 : selectedIndex - 1;
            listView.positionViewAtIndex(selectedIndex, ListView.Contain);
        }

        function selectCurrent() {
            if (selectedIndex < 0 || selectedIndex >= filteredOptions.length || selectedIndex === root.options.length) {
                return;
            }

            const val = filteredOptions[selectedIndex];
            root.valueChanged(selectedIndex, val);

            close();
        }

        onOpened: {
            selectedIndex = -1;
            if (searchField.text.length > 0) {
                initFinder();
                searchQuery = searchField.text;
            } else {
                fzfFinder = null;
                searchQuery = "";
            }
        }

        onClosed: root.isEditing = false

        parent: Overlay.overlay
        width: root.popupWidth === -1 ? undefined : (root.popupWidth > 0 ? root.popupWidth : (dropdown.width + root.popupWidthOffset))
        height: {
            let h = root.enableFuzzySearch ? 54 : 0;
            if (root.options.length === 0 && root.emptyText !== "") {
                h += 32;
            } else {
                h += Math.min(filteredOptions.length, 10) * 36;
            }
            return Math.min(root.maxPopupHeight, h + 16);
        }
        padding: 0
        modal: true
        dim: false
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: "transparent"
        }

        contentItem: Rectangle {
            id: contentSurface

            LayoutMirroring.enabled: I18n.isRtl
            LayoutMirroring.childrenInherit: true
            color: Qt.rgba(Theme.surfaceContainer.r, Theme.surfaceContainer.g, Theme.surfaceContainer.b, 1)
            border.color: Theme.primary
            border.width: 2
            radius: Theme.cornerRadius

            ElevationShadow {
                id: shadowLayer
                anchors.fill: parent
                z: -1
                level: Theme.elevationLevel2
                fallbackOffset: 4
                targetRadius: contentSurface.radius
                targetColor: contentSurface.color
                borderColor: contentSurface.border.color
                borderWidth: contentSurface.border.width
                shadowEnabled: Theme.elevationEnabled && SettingsData.popoutElevationEnabled
            }

            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingS

                Rectangle {
                    id: searchContainer

                    width: parent.width
                    height: 42
                    visible: root.enableFuzzySearch
                    radius: Theme.cornerRadius
                    color: root.usePopupTransparency ? Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency) : Theme.surfaceContainerHigh

                    DankTextField {
                        id: searchField

                        anchors.fill: parent
                        anchors.margins: 1
                        placeholderText: I18n.tr("Search...")
                        topPadding: Theme.spacingS
                        bottomPadding: Theme.spacingS
                        onTextChanged: searchDebounce.restart()
                        Keys.onDownPressed: dropdownMenu.selectNext()
                        Keys.onUpPressed: dropdownMenu.selectPrevious()
                        Keys.onReturnPressed: dropdownMenu.selectCurrent()
                        Keys.onEnterPressed: dropdownMenu.selectCurrent()
                        Keys.onPressed: event => {
                            if (!(event.modifiers & Qt.ControlModifier))
                                return;
                            switch (event.key) {
                            case Qt.Key_N:
                            case Qt.Key_J:
                                dropdownMenu.selectNext();
                                event.accepted = true;
                                break;
                            case Qt.Key_P:
                            case Qt.Key_K:
                                dropdownMenu.selectPrevious();
                                event.accepted = true;
                                break;
                            }
                        }

                        Timer {
                            id: searchDebounce
                            interval: 50
                            onTriggered: {
                                if (!dropdownMenu.fzfFinder)
                                    dropdownMenu.initFinder();
                                dropdownMenu.searchQuery = searchField.text;
                                dropdownMenu.selectedIndex = -1;
                            }
                        }
                    }
                }

                Item {
                    width: 1
                    height: Theme.spacingXS
                    visible: root.enableFuzzySearch
                }

                Item {
                    width: parent.width
                    height: 32
                    visible: root.options.length === 0 && root.emptyText !== ""

                    StyledText {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.leftMargin: Theme.spacingS
                        anchors.rightMargin: Theme.spacingS
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.emptyText
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.surfaceVariantText
                        horizontalAlignment: Text.AlignLeft
                    }
                }

                DankListView {
                    id: listView

                    width: parent.width
                    height: parent.height - (root.enableFuzzySearch ? searchContainer.height + Theme.spacingXS : 0) - (root.options.length === 0 && root.emptyText !== "" ? 32 : 0)
                    clip: true
                    model: ScriptModel {
                        values: dropdownMenu.filteredOptions
                    }
                    spacing: 2

                    interactive: true
                    flickDeceleration: 1500
                    maximumFlickVelocity: 2000
                    boundsBehavior: Flickable.DragAndOvershootBounds
                    boundsMovement: Flickable.FollowBoundsBehavior
                    pressDelay: 0
                    flickableDirection: Flickable.VerticalFlick

                    delegate: Rectangle {
                        id: delegateRoot

                        required property var modelData
                        required property int index
                        property bool isAddNew: index === root.options.length
                        property bool isSelected: dropdownMenu.selectedIndex === index
                        property bool isCurrentValue: root.currentIdx === index
                        property string iconName: isAddNew ? "add" : (root.optionIconMap[modelData] ?? "")

                        width: ListView.view.width
                        height: 32
                        radius: Theme.cornerRadius
                        color: isSelected ? Theme.primaryHover : optionArea.containsMouse ? Theme.primaryHoverLight : "transparent"

                        DankTextField {
                            id: inlineEdit
                            anchors.fill: parent
                            visible: root.isEditing && delegateRoot.isAddNew
                            focus: visible
                            placeholderText: root.addNewTextEditPlaceholder
                            onAccepted: {
                                if (text.trim() !== "") {
                                    root.valueAdded(delegateRoot.index, text);
                                }
                                root.isEditing = false;
                                text = "";
                            }
                            Keys.onEscapePressed: {
                                root.isEditing = false;
                                text = "";
                            }
                        }

                        Row {
                            visible: !(root.isEditing && delegateRoot.isAddNew)
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: Theme.spacingS
                            anchors.rightMargin: Theme.spacingS
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: Theme.spacingS

                            DankIcon {
                                name: delegateRoot.iconName
                                size: 18
                                color: delegateRoot.isCurrentValue || delegateRoot.isAddNew ? Theme.primary : Theme.surfaceText
                                visible: name !== ""
                            }

                            StyledText {
                                anchors.verticalCenter: parent.verticalCenter
                                text: delegateRoot.modelData
                                font.pixelSize: Theme.fontSizeMedium
                                color: delegateRoot.isCurrentValue ? Theme.primary : Theme.surfaceText
                                font.weight: delegateRoot.isCurrentValue ? Font.Medium : Font.Normal
                                width: root.popupWidth > 0 ? undefined : (delegateRoot.width - parent.x - Theme.spacingS * 2)
                                elide: root.popupWidth > 0 ? Text.ElideNone : Text.ElideRight
                                wrapMode: Text.NoWrap
                                horizontalAlignment: Text.AlignLeft
                            }
                        }

                        DankIcon {
                            id: deleteBtn
                            anchors.right: parent.right
                            anchors.rightMargin: Theme.spacingS
                            anchors.verticalCenter: parent.verticalCenter
                            name: "delete"
                            size: 18
                            color: Theme.error
                            visible: !delegateRoot.isAddNew && optionArea.containsMouse
                        }

                        MouseArea {
                            id: optionArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                let deletePos = deleteBtn.mapToItem(optionArea, 0, 0);
                                let isDeleteClicked = mouseX >= deletePos.x && mouseX <= (deletePos.x + deleteBtn.width) && mouseY >= deletePos.y && mouseY <= (deletePos.y + deleteBtn.height);

                                if (isDeleteClicked && !delegateRoot.isAddNew) {
                                    const deletedIndex = delegateRoot.index;
                                    const lastIdx = root.options.length - 1;

                                    root.valueDeleted(deletedIndex, root.options[deletedIndex]);

                                    // there's only 1 item, so we deleted the last one
                                    if (lastIdx <= 0) {
                                        root.valueChanged(-1, "");
                                        return;
                                    }

                                    const deletedSelf = deletedIndex === root.currentIdx;
                                    if (deletedSelf) {
                                        if (root.currentIdx === lastIdx) {
                                            const nextIdx = deletedIndex - 1;
                                            root.valueChanged(nextIdx, root.options[nextIdx] ?? "");
                                        } else if (root.currentIdx < lastIdx) {
                                            const nextIdx = root.currentIdx;
                                            root.valueChanged(nextIdx, root.options[nextIdx]);
                                        }
                                    } else if (deletedIndex < root.currentIdx) {
                                        let nextIdx = Math.max(0, root.currentIdx - 1);
                                        root.valueChanged(nextIdx, root.options[nextIdx]);
                                    } else {
                                        root.valueChanged(0, root.options[0]);
                                    }

                                    return;
                                }

                                if (delegateRoot.isAddNew) {
                                    root.isEditing = true;
                                    Qt.callLater(inlineEdit.forceActiveFocus);
                                } else {
                                    root.valueChanged(delegateRoot.index, delegateRoot.modelData);
                                    root.closeDropdownMenu();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
