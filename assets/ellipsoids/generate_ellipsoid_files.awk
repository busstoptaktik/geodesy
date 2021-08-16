# Generate Rust Geodesy Ellipsoid definition files from
# PROJ ellipsoid list.
#
# Usage:
#    proj -le | tr =\t "  " | awk -f generate_ellipsoid_files.awk
#
# Thomas Knudsen, 2021-08-16

{
    mnemonic = $1
    size_designation = $2
    size_value = $3
    shape_designation = $4
    shape_value = $5

    # Size parameter must be a
    if (size_designation != "a") {
        print "Aaargh - " mnemonic
        next
    }

    # Shape parameter may be b or rf
    if (shape_designation == "b") {
        rf = 0
        if (size_value != shape_value) {
            rf = size_value / (size_value - shape_value)
        }
    } else if (shape_designation == "rf") {
        rf = $5
    } else {
        print "Aaargh - " mnemonic
        next
    }

    # Dig out the description
    $1 = $2 = $3 = $4 = $5 = ""
    gsub(/^[ \t]+/,"", $0)
    description = $0

    # And write the file
    filename = mnemonic ".yml"
    print "ellipsoid:"                                  >filename
    print "    description: " description               >filename
    print "    mnemonic: " mnemonic                     >filename
    print "    shortcut: "                              >filename
    print "        a: " size_value                      >filename
    print "        rf: " rf                             >filename
    print "    definition:"                             >filename
    print "        a: " size_value                      >filename
    print "        " shape_designation ": " shape_value >filename
}
