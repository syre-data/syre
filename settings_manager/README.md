# Settings Manager

> Crate for managing settings.

## Settings

This is the base trait required to implement all other setting types. Settings
objects lock their file while they exist so other processes can not modify them 

## System Settings

Represents settings that are used system wide. i.e. These represent settings
that should be singletons.

## Local Settings

Represents settings that are local to a specific resource. i.e. Multiple
instances of this type of settings can exist at once.

## List Setting

Represents a setting that is a simple list of objects.
