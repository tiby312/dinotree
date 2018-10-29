# memory usage

~~~~text
x=object (without its aabb)
i=index of an aabb object
r=aabb



xxxxx ->user provides bots

xxxxx iririririr -> the inner tree of (index,aabb) elements is created and sorted. 

xxxxx iririririr xrxrxrxrxr -> the inner tree is used to create the dinotree. 

xxxxx xrxrxrxrxr iririririr iiiii ->the indicies of the bots in generated (so that we know the indicies of where to move the bots back to)

xxxxx xrxrxrxrxr iiiii ->remove inner tree. Now the tree is setup and ready to be used. The user can call apply() to apply changes to the right bots (by using the index list stored).

xxxxx -> dinotree is destroyed leaving just the original slice.

~~~~

