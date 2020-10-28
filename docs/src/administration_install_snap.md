# Snap Installation

## Normal Installation

To install Lemmy as a snap, run this command:

    sudo snap install lemmy

You will then have a running Lemmy instance.

## Configuration

To configure your new Lemmy instance, see the [administration configuration documentation](administration_configuration.md).

## Manual Installation From Source

To build and install the snap yourself, run these commands:

    SNAPCRAFT_BUILD_ENVIRONMENT_MEMORY=4G SNAPCRAFT_BUILD_ENVIRONMENT_CPU=4 snapcraft
    sudo snap install lemmy*.snap --dangerous --devmode
