#include <stdio.h>

float geoid[11][21];
float datum[11][21][2];

int main(void) {
    int i, j;
    for (i = 0; i < 11; i++) for (j = 0; j < 21; j++) {
         geoid[i][j] = 60. - i + (0. + j) / 100.;
         datum[i][j][0] = 0 + j;
         datum[i][j][1] = 60. - i;
    }

    FILE *pile = fopen("pile.bin", "wb");
    fwrite(&geoid, sizeof(float), 11*21, pile);
    fwrite(&datum, sizeof(float), 11*21*2, pile);
    printf("%f\n%f\n", (1.0/0.0), (1.0/0.0)/0.0);
}
